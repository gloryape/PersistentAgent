"""Check what runs exist in the parquet data and which is current."""
import sys, io
sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8', errors='replace')

import polars as pl
import glob
import os
from collections import defaultdict

os.chdir(os.path.dirname(os.path.abspath(__file__)))
os.chdir('..')

files = sorted(glob.glob('data/metrics/*.parquet'))
print(f"Total parquet files: {len(files)}")

# Check the last 10 files
print("\n=== LATEST 10 PARQUET FILES ===")
for f in files[-10:]:
    try:
        d = pl.read_parquet(f)
        run_id = d['run_id'][0] if 'run_id' in d.columns else 'N/A'
        tick_min = d['tick'].min()
        tick_max = d['tick'].max()
        stiff = d['stiffness'].mean()
        eff = d['efficiency'].mean()
        print(f"  {os.path.basename(f)}: run={run_id[:16]}... ticks={tick_min}-{tick_max} stiff={stiff:.3f} eff={eff:.4f}")
    except Exception as e:
        print(f"  {os.path.basename(f)}: ERROR {e}")

# Count files per run_id
print("\n=== RUN IDS (from last 200 files) ===")
run_counts = defaultdict(int)
run_ticks = defaultdict(lambda: [999999999, 0])
run_stiffness = defaultdict(list)
for f in files[-200:]:
    try:
        d = pl.read_parquet(f)
        if 'run_id' in d.columns:
            rid = d['run_id'][0]
            run_counts[rid] += 1
            tmin = d['tick'].min()
            tmax = d['tick'].max()
            run_ticks[rid][0] = min(run_ticks[rid][0], tmin)
            run_ticks[rid][1] = max(run_ticks[rid][1], tmax)
            run_stiffness[rid].append(d['stiffness'].mean())
    except:
        pass

for rid, count in sorted(run_counts.items(), key=lambda x: -x[1]):
    trange = run_ticks[rid]
    avg_stiff = sum(run_stiffness[rid]) / len(run_stiffness[rid])
    print(f"  {rid[:20]}...: {count} files, ticks {trange[0]}-{trange[1]}, avg_stiff={avg_stiff:.3f}")

# Check checkpoint file
print("\n=== CHECKPOINT STATE ===")
ckpt_path = 'data/checkpoint/state.bin'
if os.path.exists(ckpt_path):
    size = os.path.getsize(ckpt_path)
    mtime = os.path.getmtime(ckpt_path)
    import datetime
    mod_time = datetime.datetime.fromtimestamp(mtime)
    print(f"  state.bin exists: {size:,} bytes, modified {mod_time}")
else:
    print("  No checkpoint file found")

# Check save files
saves_dir = 'data/saves'
if os.path.exists(saves_dir):
    saves = os.listdir(saves_dir)
    print(f"\n=== SAVE FILES ({len(saves)}) ===")
    for s in sorted(saves)[-5:]:
        p = os.path.join(saves_dir, s)
        print(f"  {s}: {os.path.getsize(p):,} bytes")

# Check if organism is currently running (look at terminal output time)
print("\n=== CURRENT TERMINAL (last line check) ===")
term_file = None  # Terminal file path is IDE-specific; skip if not available
if term_file and os.path.exists(term_file):
    with open(term_file, 'r', encoding='utf-8', errors='replace') as tf:
        lines = tf.readlines()
        # Find last line with timestamp
        for line in reversed(lines[-20:]):
            if '[SIM]' in line:
                print(f"  {line.strip()[:120]}")
                break
else:
    print("  (terminal file not configured)")
