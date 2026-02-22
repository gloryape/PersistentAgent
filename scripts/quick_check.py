"""Quick metrics check for the CURRENT run (by run_id prefix in filename)."""
import sys, io
sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8', errors='replace')

import polars as pl
import glob
import os
import json

os.chdir(os.path.dirname(os.path.abspath(__file__)))
os.chdir('..')

all_files = glob.glob('data/metrics/*.parquet')
print(f"Total parquet files: {len(all_files)}")

if not all_files:
    print("No parquet files found!")
    exit()

# Identify current run from the MOST RECENTLY MODIFIED file (not alphabetical order).
all_files_with_mtime = [(f, os.path.getmtime(f)) for f in all_files]
all_files_with_mtime.sort(key=lambda x: x[1], reverse=True)  # newest first
most_recent_path = all_files_with_mtime[0][0]
latest_name = os.path.basename(most_recent_path)
# Format: run_UUID_epoch_NNNNNN.parquet  ->  extract run_UUID part
parts = latest_name.split('_epoch_')
run_prefix = parts[0] if len(parts) == 2 else '_'.join(latest_name.split('_')[:5])
print(f"Current run prefix: {run_prefix} (from most recently modified file)")

# Read only files from this run
run_files = [f for f in all_files if os.path.basename(f).startswith(run_prefix)]
run_files_with_mtime = [(f, os.path.getmtime(f)) for f in run_files]
run_files_with_mtime.sort(key=lambda x: x[1])
run_files = [f for f, _ in run_files_with_mtime]
print(f"Files for this run: {len(run_files)}")

all_dfs = []
for f in run_files:
    try:
        all_dfs.append(pl.read_parquet(f))
    except Exception:
        pass

df = pl.concat(all_dfs)
print(f"Total rows: {len(df):,}")
print(f"Tick range: {df['tick'].min()} to {df['tick'].max()}")
print(f"Engine hours: {df['engine_hours'].min():.6f} to {df['engine_hours'].max():.6f}")
duration_s = (df['engine_hours'].max() - df['engine_hours'].min()) * 3600
print(f"Duration: {duration_s:.1f} seconds ({duration_s/60:.2f} minutes)")

# Detect multi-agent
has_agent_id = 'agent_id' in df.columns
if has_agent_id:
    agent_ids = sorted(df['agent_id'].unique().to_list())
    n_agents = len(agent_ids)
else:
    agent_ids = ['agent_0']
    n_agents = 1
print(f"Agents detected: {n_agents} {agent_ids}")
print()

# Helper to run a section for a subset
def section_for(label: str, sdf: pl.DataFrame):
    """Print interaction-only metrics for a given subset."""
    inter = sdf.filter(pl.col('energy_in') > 0)
    n_inter = len(inter)
    n_total = len(sdf)
    print(f"  [{label}] Rows: {n_total:,}  Interactions: {n_inter:,} ({100*n_inter/n_total:.1f}%)" if n_total else f"  [{label}] No rows")
    if n_inter == 0:
        return
    eff_inter = inter['efficiency']
    res_inter = inter['resonance']
    print(f"  [{label}] Mean eff (interactions): {eff_inter.mean():.4f}  Mean res: {res_inter.mean():.4f}")
    nq = max(1, n_inter // 4)
    inter_early = inter.head(nq)
    inter_late = inter.tail(nq)
    print(f"  [{label}] Early 25%: eff={inter_early['efficiency'].mean():.4f}  Late 25%: eff={inter_late['efficiency'].mean():.4f}")
    amp_inter = inter.filter(pl.col('efficiency') > 1.0)
    n_amp = len(amp_inter)
    if n_amp > 0:
        print(f"  [{label}] Amplification (eff>1.0): {n_amp:,}/{n_inter:,} ({100*n_amp/n_inter:.1f}%)  mean={amp_inter['efficiency'].mean():.4f}")


# --- INTERACTION-ONLY (the metric that matters) ---
inter = df.filter(pl.col('energy_in') > 0)
n_inter = len(inter)
n_total = len(df)
print("=== INTERACTION-ONLY (energy_in > 0) — headline metrics ===")
print(f"  Interaction rows: {n_inter:,} / {n_total:,} ({100*n_inter/n_total:.1f}% of rows)")
if n_inter == 0:
    print("  No interaction rows; cannot compute interaction-only metrics.")
else:
    eff_inter = inter['efficiency']
    res_inter = inter['resonance']
    print(f"  Mean efficiency (interactions only): {eff_inter.mean():.4f}")
    print(f"  Median efficiency (interactions only): {eff_inter.median():.4f}")
    print(f"  Mean resonance (interactions only):   {res_inter.mean():.4f}")
    nq = max(1, n_inter // 4)
    inter_early = inter.head(nq)
    inter_late = inter.tail(nq)
    print(f"  Early (first 25% of interactions):  eff={inter_early['efficiency'].mean():.4f}  res={inter_early['resonance'].mean():.4f}")
    print(f"  Latest (last 25% of interactions):   eff={inter_late['efficiency'].mean():.4f}  res={inter_late['resonance'].mean():.4f}")
    amp_inter = inter.filter(pl.col('efficiency') > 1.0)
    n_amp = len(amp_inter)
    print(f"  Amplification (eff>1.0) on interactions: {n_amp:,} / {n_inter:,} ({100*n_amp/n_inter:.1f}%)")
    if n_amp > 0:
        print(f"  Mean efficiency when > 1.0: {amp_inter['efficiency'].mean():.4f}")
    # Per-agent breakdown
    if n_agents > 1 and has_agent_id:
        print()
        print("  --- Per-Agent Interaction Breakdown ---")
        for aid in agent_ids:
            adf = df.filter(pl.col('agent_id') == aid)
            section_for(aid, adf)
print()

# Key metrics (all rows)
cols = ['coherence', 'efficiency', 'resonance', 'stiffness', 'energy_in', 'effective_energy']
print("=== KEY METRICS (all rows; diluted by tick samples) ===")
for col in cols:
    s = df[col]
    print(f"  {col:20s}: mean={s.mean():.4f}  std={s.std():.4f}  min={s.min():.4f}  max={s.max():.4f}")
print()

# Vehicle distribution
print("=== VEHICLE DISTRIBUTION ===")
total = len(df)
# Interaction-only vehicle mix (energy_in>0 = when agent actually moved; excludes Reanchor rest)
if n_inter > 0:
    vdist_inter = inter.group_by('vehicle').agg(pl.len().alias('count')).sort('count', descending=True)
    print("  [INTERACTION-ONLY rows — when agents actually moved]")
    for row in vdist_inter.iter_rows(named=True):
        pct = row['count'] / n_inter * 100
        print(f"    {row['vehicle']:25s}: {row['count']:>8,} ({pct:.1f}%)")
    print()
if n_agents > 1 and has_agent_id:
    # Aggregate (all rows: tick samples + interactions; Reanchor dominates b/c last_outcome persists)
    vdist = df.group_by('vehicle').agg(pl.len().alias('count')).sort('count', descending=True)
    print("  [ALL ROWS — tick samples + interactions]")
    for row in vdist.iter_rows(named=True):
        pct = row['count'] / total * 100
        print(f"    {row['vehicle']:25s}: {row['count']:>8,} ({pct:.1f}%)")
    # Per-agent
    for aid in agent_ids:
        adf = df.filter(pl.col('agent_id') == aid)
        avdist = adf.group_by('vehicle').agg(pl.len().alias('count')).sort('count', descending=True)
        atotal = len(adf)
        print(f"  [{aid}]")
        for row in avdist.iter_rows(named=True):
            pct = row['count'] / atotal * 100 if atotal > 0 else 0
            print(f"    {row['vehicle']:25s}: {row['count']:>8,} ({pct:.1f}%)")
else:
    vdist = df.group_by('vehicle').agg(pl.len().alias('count')).sort('count', descending=True)
    for row in vdist.iter_rows(named=True):
        pct = row['count'] / total * 100
        print(f"  {row['vehicle']:25s}: {row['count']:>8,} ({pct:.1f}%)")
print()

# Observer engagement impact
print("=== OBSERVER ENGAGEMENT IMPACT ===")
engagement_ticks = df.filter(pl.col('vehicle').str.contains('Saitama'))['tick'].to_list()
if engagement_ticks:
    from bisect import bisect_right
    interactions = df.filter(pl.col('energy_in') > 0).sort('tick')
    interaction_tick_list = interactions['tick'].to_list()
    interaction_eff_list = interactions['efficiency'].to_list()
    post_effs = []
    for et in engagement_ticks:
        idx = bisect_right(interaction_tick_list, et)
        if idx < len(interaction_tick_list):
            post_effs.append(interaction_eff_list[idx])
    if post_effs:
        baseline_eff = sum(interaction_eff_list) / len(interaction_eff_list)
        post_mean = sum(post_effs) / len(post_effs)
        delta = post_mean - baseline_eff
        verdict = "HELPING" if delta > 0.02 else "HINDERING" if delta < -0.02 else "NEUTRAL"
        print(f"  Post-engagement interactions: {len(post_effs)}")
        print(f"  Post-engagement mean efficiency: {post_mean:.4f}")
        print(f"  Baseline mean efficiency: {baseline_eff:.4f}")
        print(f"  Delta: {delta:+.4f} ({verdict})")
    else:
        print("  No post-engagement interactions found")
else:
    print("  No Saitama+Complement engagements in this run")
print()

# Efficiency trend
n = len(df)
q1 = df.head(n // 4)
q4 = df.tail(n // 4)
print("=== EFFICIENCY TREND (first 25% vs last 25%) ===")
print(f"  Early:  eff={q1['efficiency'].mean():.4f}  res={q1['resonance'].mean():.4f}  stiff={q1['stiffness'].mean():.4f}")
print(f"  Latest: eff={q4['efficiency'].mean():.4f}  res={q4['resonance'].mean():.4f}  stiff={q4['stiffness'].mean():.4f}")
print()

# Resonance amplification
amp = df.filter(pl.col('efficiency') > 1.0)
print(f"=== RESONANCE AMPLIFICATION (efficiency > 1.0) ===")
print(f"  Ticks with efficiency > 1.0: {len(amp)} / {len(df)} ({len(amp)/len(df)*100:.1f}%)")
if len(amp) > 0:
    print(f"  Max efficiency: {amp['efficiency'].max():.4f}")
    print(f"  Mean efficiency when > 1.0: {amp['efficiency'].mean():.4f}")
print()

# Efficiency buckets
print("=== EFFICIENCY DISTRIBUTION ===")
for lo, hi, label in [(0.0, 0.01, "Near-zero"), (0.01, 0.1, "Low"), (0.1, 0.5, "Medium"),
                       (0.5, 1.0, "High"), (1.0, 100.0, "Amplification")]:
    cnt = len(df.filter((pl.col('efficiency') >= lo) & (pl.col('efficiency') < hi)))
    print(f"  {label:15s} ({lo:.2f}-{hi:.2f}): {cnt:>8,} ({cnt/total*100:.1f}%)")
print()

# Observer state (recent)
print("=== OBSERVER (from recent 200 ticks) ===")
recent = df.tail(200)
print(f"  Recent eff: {recent['efficiency'].mean():.4f}")
print(f"  Recent res: {recent['resonance'].mean():.4f}")
print(f"  Recent stiff: {recent['stiffness'].mean():.4f}")
rvdist = recent.group_by('vehicle').agg(pl.len().alias('count')).sort('count', descending=True)
for row in rvdist.iter_rows(named=True):
    print(f"    {row['vehicle']:25s}: {row['count']}")
print()

# Movement — per-agent if multi-agent
print("=== MOVEMENT ===")
if n_agents > 1 and has_agent_id:
    for aid in agent_ids:
        adf = df.filter(pl.col('agent_id') == aid)
        print(f"  [{aid}] X: {adf['location_x'].min()} to {adf['location_x'].max()}  Y: {adf['location_y'].min()} to {adf['location_y'].max()}")
        voxels = adf.select([
            (pl.col('location_x') / 10).cast(pl.Int32).alias('vx'),
            (pl.col('location_y') / 10).cast(pl.Int32).alias('vy'),
        ]).unique()
        print(f"  [{aid}] Unique voxels visited: {len(voxels)}")
    # Also show shared voxels (overlap between agents)
    per_agent_voxels = {}
    for aid in agent_ids:
        adf = df.filter(pl.col('agent_id') == aid)
        vs = set(
            adf.select([
                (pl.col('location_x') / 10).cast(pl.Int32).alias('vx'),
                (pl.col('location_y') / 10).cast(pl.Int32).alias('vy'),
            ]).unique().iter_rows()
        )
        per_agent_voxels[aid] = vs
    if len(per_agent_voxels) >= 2:
        all_sets = list(per_agent_voxels.values())
        shared = all_sets[0]
        for s in all_sets[1:]:
            shared = shared & s
        print(f"  Shared voxels (visited by ALL agents): {len(shared)}")
else:
    print(f"  X range: {df['location_x'].min()} to {df['location_x'].max()}")
    print(f"  Y range: {df['location_y'].min()} to {df['location_y'].max()}")
    voxels = df.select([
        (pl.col('location_x') / 10).cast(pl.Int32).alias('vx'),
        (pl.col('location_y') / 10).cast(pl.Int32).alias('vy'),
    ]).unique()
    print(f"  Unique voxels visited: {len(voxels)}")

# Return visits
if n_inter >= 2:
    if 'voxel_x' in inter.columns and 'voxel_y' in inter.columns:
        inter_v = inter.with_columns([
            pl.col('voxel_x').cast(pl.Int32).alias('vx'),
            pl.col('voxel_y').cast(pl.Int32).alias('vy'),
        ])
    else:
        inter_v = inter.with_columns([
            (pl.col('location_x') / 10).cast(pl.Int32).alias('vx'),
            (pl.col('location_y') / 10).cast(pl.Int32).alias('vy'),
        ])
    inter_v = inter_v.filter(pl.col('vx').is_not_null() & pl.col('vy').is_not_null())
    n_with_voxel = len(inter_v)
    n_unknown_voxel = n_inter - n_with_voxel
    n_vox_inter = inter_v.select(['vx', 'vy']).unique().height
    inter_v = inter_v.with_columns(
        pl.col('tick').rank('ordinal').over(['vx', 'vy']).alias('visit_num')
    )
    first_visit = inter_v.filter(pl.col('visit_num') == 1)
    return_visit = inter_v.filter(pl.col('visit_num') > 1)
    n_first = len(first_visit)
    n_return = len(return_visit)
    print()
    print("=== RETURN VISITS (Reflection Axiom: memory rewards revisit) ===")
    if n_unknown_voxel > 0:
        print(f"  (excluded {n_unknown_voxel:,} interaction rows with unknown voxel)")
    print(f"  Unique voxels with >= 1 interaction: {n_vox_inter:,}")
    print(f"  First-visit interactions:  {n_first:,}  mean eff={first_visit['efficiency'].mean():.4f}  res={first_visit['resonance'].mean():.4f}")
    if n_return > 0:
        print(f"  Return-visit interactions: {n_return:,}  mean eff={return_visit['efficiency'].mean():.4f}  res={return_visit['resonance'].mean():.4f}")
        q25 = return_visit['resonance'].quantile(0.25)
        q50 = return_visit['resonance'].quantile(0.50)
        q75 = return_visit['resonance'].quantile(0.75)
        rv_q = return_visit.with_columns(
            pl.when(pl.col('resonance') <= q25).then(1)
            .when(pl.col('resonance') <= q50).then(2)
            .when(pl.col('resonance') <= q75).then(3)
            .otherwise(4)
            .alias('quartile')
        )
        q_agg = rv_q.group_by('quartile').agg([
            pl.col('efficiency').mean().alias('eff'),
            pl.col('resonance').mean().alias('res'),
            pl.len().alias('n'),
        ]).sort('quartile')
        print("  Return-visit efficiency by resonance quartile:")
        labels = {1: "Q1 (low res)", 2: "Q2", 3: "Q3", 4: "Q4 (high res)"}
        for row in q_agg.iter_rows(named=True):
            q, eff, res, n_r = row['quartile'], row['eff'], row['res'], row['n']
            print(f"    {labels.get(q, f'Q{q}'):14s}:  eff={eff:.4f}  res={res:.4f}  ({n_r:,} rows)")
    else:
        print(f"  Return-visit interactions: 0 (no voxel revisited yet)")

    # Per-agent return visit comparison (do agents benefit from each other's deposits?)
    if n_agents > 1 and has_agent_id and 'agent_id' in inter_v.columns:
        print()
        print("  --- Cross-Agent Return Visits ---")
        for aid in agent_ids:
            a_inter = inter_v.filter(pl.col('agent_id') == aid)
            a_return = a_inter.filter(pl.col('visit_num') > 1)
            a_total = len(a_inter)
            a_ret = len(a_return)
            if a_total > 0:
                ret_pct = 100 * a_ret / a_total
                eff_str = f"  eff={a_return['efficiency'].mean():.4f}" if a_ret > 0 else ""
                print(f"  [{aid}] {a_ret:,}/{a_total:,} return visits ({ret_pct:.1f}%){eff_str}")
else:
    print()
    print("=== RETURN VISITS ===")
    print("  (need at least 2 interaction rows)")

# Proprioceptive stimuli
logs_dir = 'data/logs'
jsonl_files = glob.glob(os.path.join(logs_dir, run_prefix + '_epoch_*.jsonl'))
proprio_events = []
for jf in jsonl_files:
    try:
        with open(jf, 'r', encoding='utf-8', errors='replace') as f:
            for line in f:
                line = line.strip()
                if not line or 'proprio' not in line.lower():
                    continue
                try:
                    rec = json.loads(line)
                    if rec.get('event_type') == 'proprio':
                        proprio_events.append(rec)
                except (json.JSONDecodeError, TypeError):
                    pass
    except Exception:
        pass
proprio_count = len(proprio_events)
print()
print("=== PROPRIOCEPTIVE STIMULI (from Stream B JSONL) ===")
if proprio_count > 0:
    print(f"  Total PROPRIO events: {proprio_count:,}")
    coverages = [e.get('data', {}).get('coverage') for e in proprio_events if isinstance(e.get('data'), dict)]
    contrasts = [e.get('data', {}).get('scan_contrast', e.get('data', {}).get('directional_contrast')) for e in proprio_events if isinstance(e.get('data'), dict)]
    coverages = [x for x in coverages if x is not None]
    contrasts = [x for x in contrasts if x is not None]
    saliences = [e.get('data', {}).get('salience') for e in proprio_events if isinstance(e.get('data'), dict)]
    novelties = [e.get('data', {}).get('novelty') for e in proprio_events if isinstance(e.get('data'), dict)]
    saliences = [x for x in saliences if x is not None]
    novelties = [x for x in novelties if x is not None]
    if coverages:
        print(f"  Mean coverage: {sum(coverages)/len(coverages):.4f}")
    if contrasts:
        print(f"  Mean scan_contrast: {sum(contrasts)/len(contrasts):.4f}")
    if saliences:
        print(f"  Mean salience: {sum(saliences)/len(saliences):.4f}")
    else:
        print(f"  Mean salience: (not logged — pre-salience-floor binary)")
    if novelties:
        print(f"  Mean novelty: {sum(novelties)/len(novelties):.4f}")
    tick_span = float(df['tick'].max() - df['tick'].min()) if len(df) > 0 else 0
    if tick_span > 0:
        per_100 = proprio_count / (tick_span / 100.0)
        print(f"  Frequency: {per_100:.2f} events per 100 ticks")
else:
    print("  No PROPRIO entries in JSONL.")
    print("  (Proprioceptive stimuli are logged when the field has visible structure; run with ENTROPY_DECAY=0.01 so trails persist.)")

# Build version info from JSONL
version_events = []
for jf in jsonl_files:
    try:
        with open(jf, 'r', encoding='utf-8', errors='replace') as f:
            for line in f:
                line = line.strip()
                if not line or 'version' not in line.lower():
                    continue
                try:
                    rec = json.loads(line)
                    if rec.get('event_type') == 'version':
                        version_events.append(rec)
                        break
                except (json.JSONDecodeError, TypeError):
                    pass
        if version_events:
            break
    except Exception:
        pass

print()
print("=== BUILD INFO ===")
if version_events:
    v = version_events[0].get('data', {})
    version = v.get('version', 'unknown')
    git_hash = v.get('git_hash', 'unknown')
    build_time = v.get('build_time', 'unknown')
    print(f"  Version: v{version}")
    print(f"  Commit: {git_hash}")
    print(f"  Built: {build_time}")
else:
    print("  (No version info — pre-version-logging binary)")

# Somatic motor state
if 'efficiency_momentum' in df.columns:
    em = df['efficiency_momentum']
    print()
    print("=== SOMATIC MOTOR STATE ===")
    print(f"  efficiency_momentum: mean={em.mean():.4f}  min={em.min():.4f}  max={em.max():.4f}")
    high = em.filter(em > 0.6).len()
    low = em.filter(em < 0.3).len()
    mid = len(em) - high - low
    print(f"  High commitment (>0.6): {high:,} ({100*high/len(em):.1f}%)")
    print(f"  Mid range (0.3-0.6):    {mid:,} ({100*mid/len(em):.1f}%)")
    print(f"  Exploring (<0.3):       {low:,} ({100*low/len(em):.1f}%)")

    interaction_rows = df.filter(pl.col('energy_in') > 0)
    tick_sample_rows = df.filter(pl.col('energy_in') == 0)
    if len(interaction_rows) > 0:
        em_int = interaction_rows['efficiency_momentum']
        print(f"  [DIAG] Interaction rows ({len(interaction_rows):,}): em mean={em_int.mean():.4f}  min={em_int.min():.4f}  max={em_int.max():.4f}")
    if len(tick_sample_rows) > 0:
        em_ts = tick_sample_rows['efficiency_momentum']
        print(f"  [DIAG] Tick-sample rows ({len(tick_sample_rows):,}): em mean={em_ts.mean():.4f}  min={em_ts.min():.4f}  max={em_ts.max():.4f}")

    n = len(df)
    q1_em = df.head(n // 4)['efficiency_momentum']
    q4_em = df.tail(n // 4)['efficiency_momentum']
    print(f"  [DIAG] Early (first 25%): em mean={q1_em.mean():.4f}")
    print(f"  [DIAG] Late  (last  25%): em mean={q4_em.mean():.4f}")

    zero_rows = em.filter(em == 0.0).len()
    print(f"  [DIAG] Rows with em == 0.0 exactly: {zero_rows:,}")

    if len(interaction_rows) > 0:
        sample = interaction_rows.head(20).select(['tick', 'efficiency', 'efficiency_momentum'])
        print(f"  [DIAG] First 20 interaction rows (tick, eff, em):")
        for row in sample.iter_rows(named=True):
            print(f"    tick={row['tick']:5d}  eff={row['efficiency']:.4f}  em={row['efficiency_momentum']:.4f}")

    # Per-agent somatic state
    if n_agents > 1 and has_agent_id:
        print()
        print("  --- Per-Agent Somatic State ---")
        for aid in agent_ids:
            adf = df.filter(pl.col('agent_id') == aid)
            if 'efficiency_momentum' in adf.columns:
                aem = adf['efficiency_momentum']
                a_high = aem.filter(aem > 0.6).len()
                a_total = len(aem)
                pct = 100 * a_high / a_total if a_total > 0 else 0
                print(f"  [{aid}] em mean={aem.mean():.4f}  high(>0.6): {pct:.1f}%")

elif 'efficiency_accumulator' in df.columns and 'motor_regime' in df.columns:
    acc = df['efficiency_accumulator']
    print()
    print("=== MOTOR REGIME (phase transition — legacy) ===")
    print(f"  efficiency_accumulator: mean={acc.mean():.4f}  min={acc.min():.4f}  max={acc.max():.4f}")
    rdist = df.group_by('motor_regime').agg(pl.len().alias('count')).sort('count', descending=True)
    for row in rdist.iter_rows(named=True):
        pct = row['count'] / len(df) * 100
        print(f"  motor_regime {row['motor_regime']!r}: {row['count']:,} ({pct:.1f}%)")

# Stiffness reality check
print()
print("=== STIFFNESS CHECK (new K0=0.5 vs old K0=2.0) ===")
print(f"  Mean stiffness: {df['stiffness'].mean():.4f}")
print(f"  Median stiffness: {df['stiffness'].median():.4f}")
print(f"  (Old run had mean ~20.3. If this is <<20, new physics is active)")
