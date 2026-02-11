"""Quick metrics check for the CURRENT run (by run_id prefix in filename)."""
import sys, io
sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8', errors='replace')

import polars as pl
import glob
import os

os.chdir(os.path.dirname(os.path.abspath(__file__)))
os.chdir('..')

files = sorted(glob.glob('data/metrics/*.parquet'))
print(f"Total parquet files: {len(files)}")

if not files:
    print("No parquet files found!")
    exit()

# Identify current run from the latest file's name prefix
latest_name = os.path.basename(files[-1])
# Format: run_UUID_epoch_NNNNNN.parquet  ->  extract run_UUID part
parts = latest_name.split('_epoch_')
run_prefix = parts[0] if len(parts) == 2 else '_'.join(latest_name.split('_')[:5])
print(f"Current run prefix: {run_prefix}")

# Read only files from this run
run_files = [f for f in files if os.path.basename(f).startswith(run_prefix)]
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
print()

# Key metrics
cols = ['coherence', 'efficiency', 'resonance', 'stiffness', 'energy_in', 'effective_energy']
print("=== KEY METRICS ===")
for col in cols:
    s = df[col]
    print(f"  {col:20s}: mean={s.mean():.4f}  std={s.std():.4f}  min={s.min():.4f}  max={s.max():.4f}")
print()

# Vehicle distribution
print("=== VEHICLE DISTRIBUTION ===")
vdist = df.group_by('vehicle').agg(pl.len().alias('count')).sort('count', descending=True)
total = len(df)
for row in vdist.iter_rows(named=True):
    pct = row['count'] / total * 100
    print(f"  {row['vehicle']:25s}: {row['count']:>8,} ({pct:.1f}%)")
print()

# Efficiency trend: first 25% vs last 25%
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

# Observer state
print("=== OBSERVER (from recent 200 ticks) ===")
recent = df.tail(200)
print(f"  Recent eff: {recent['efficiency'].mean():.4f}")
print(f"  Recent res: {recent['resonance'].mean():.4f}")
print(f"  Recent stiff: {recent['stiffness'].mean():.4f}")
rvdist = recent.group_by('vehicle').agg(pl.len().alias('count')).sort('count', descending=True)
for row in rvdist.iter_rows(named=True):
    print(f"    {row['vehicle']:25s}: {row['count']}")
print()

# Movement
print("=== MOVEMENT ===")
print(f"  X range: {df['location_x'].min()} to {df['location_x'].max()}")
print(f"  Y range: {df['location_y'].min()} to {df['location_y'].max()}")
voxels = df.select([
    (pl.col('location_x') / 10).cast(pl.Int32).alias('vx'),
    (pl.col('location_y') / 10).cast(pl.Int32).alias('vy'),
]).unique()
print(f"  Unique voxels visited: {len(voxels)}")

# Stiffness reality check
print()
print("=== STIFFNESS CHECK (new K0=0.5 vs old K0=2.0) ===")
print(f"  Mean stiffness: {df['stiffness'].mean():.4f}")
print(f"  Median stiffness: {df['stiffness'].median():.4f}")
print(f"  (Old run had mean ~20.3. If this is <<20, new physics is active)")
