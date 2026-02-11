#!/usr/bin/env python3
"""Analyze parquet metrics from Quaternity Organism runs."""

import pandas as pd
import os
import glob
from pathlib import Path
from datetime import datetime
import sys

METRICS_DIR = Path(__file__).resolve().parent.parent / "data" / "metrics"

print("=" * 80)
print("QUATERNITY ORGANISM - METRICS ANALYSIS REPORT")
print(f"Generated: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
print("=" * 80)

# 1. Discover all parquet files
parquet_files = list(METRICS_DIR.glob("*.parquet"))
print(f"\nTotal parquet files: {len(parquet_files)}")

file_info = []
for f in parquet_files:
    stat = f.stat()
    stem = f.stem
    parts = stem.split("_epoch_")
    run_id = parts[0] if len(parts) == 2 else stem
    epoch = int(parts[1]) if len(parts) == 2 else -1
    file_info.append({
        "path": str(f),
        "run_id": run_id,
        "epoch": epoch,
        "mtime": stat.st_mtime,
        "size": stat.st_size,
    })

fi_df = pd.DataFrame(file_info)

# 2. Run summary
print("\n" + "=" * 80)
print("SECTION 1: RUN SUMMARY")
print("=" * 80)
for rid, grp in fi_df.groupby("run_id"):
    latest = datetime.fromtimestamp(grp["mtime"].max()).strftime("%Y-%m-%d %H:%M:%S")
    total_kb = grp["size"].sum() / 1024
    print(f"  {rid}")
    print(f"    Epochs: {len(grp)}, Last modified: {latest}, Size: {total_kb:.1f} KB")

# 3. Load most recent run
most_recent_file = fi_df.loc[fi_df["mtime"].idxmax()]
most_recent_run = most_recent_file["run_id"]
recent_files = fi_df[fi_df["run_id"] == most_recent_run].sort_values("epoch")

print(f"\n{'=' * 80}")
print(f"SECTION 2: ANALYZING MOST RECENT RUN")
print(f"Run ID: {most_recent_run}")
print(f"Epoch files: {len(recent_files)}")
print(f"{'=' * 80}")

dfs = []
for _, row in recent_files.iterrows():
    try:
        df_part = pd.read_parquet(row["path"])
        dfs.append(df_part)
    except Exception as e:
        print(f"  ERROR reading epoch {row['epoch']}: {e}")

df = pd.concat(dfs, ignore_index=True)
if "tick" in df.columns:
    df = df.sort_values("tick").reset_index(drop=True)

# 4. Schema
print(f"\nColumns ({len(df.columns)}):")
for col in df.columns:
    print(f"  {col:30s} dtype={df[col].dtype}")

# 5. Basic stats
print(f"\nTotal rows (ticks): {len(df):,}")

# 6. Time span
print(f"\n{'=' * 80}")
print("SECTION 3: TIME SPAN")
print("=" * 80)
if "tick" in df.columns:
    print(f"  Tick range: {df['tick'].min()} to {df['tick'].max()} ({df['tick'].max() - df['tick'].min()} ticks)")
if "timestamp_ms" in df.columns:
    t0 = df["timestamp_ms"].min()
    t1 = df["timestamp_ms"].max()
    duration_s = (t1 - t0) / 1000.0
    print(f"  Duration: {duration_s:.1f} seconds ({duration_s/60:.2f} minutes)")
if "engine_hours" in df.columns:
    print(f"  Engine hours: {df['engine_hours'].min():.6f} to {df['engine_hours'].max():.6f}")

# 7. Key metrics summary
print(f"\n{'=' * 80}")
print("SECTION 4: KEY METRICS SUMMARY")
print("=" * 80)
key_metrics = ["coherence", "efficiency", "resonance", "stiffness", "phase",
               "energy_in", "effective_energy"]
available = [m for m in key_metrics if m in df.columns]
missing = [m for m in key_metrics if m not in df.columns]
if missing:
    print(f"  (Missing: {', '.join(missing)})")
if available:
    print(df[available].describe().round(6).to_string())

# 8. Location & movement
print(f"\n{'=' * 80}")
print("SECTION 5: MOVEMENT & SPATIAL EXPLORATION")
print("=" * 80)
loc_cols = [c for c in df.columns if "location" in c.lower() or "loc_" in c.lower()]
for col in loc_cols:
    if pd.api.types.is_numeric_dtype(df[col]):
        mn, mx = df[col].min(), df[col].max()
        print(f"  {col}: min={mn:.2f}, max={mx:.2f}, range={mx-mn:.2f}")

voxel_cols = [c for c in df.columns if "voxel" in c.lower()]
for col in voxel_cols:
    if pd.api.types.is_numeric_dtype(df[col]):
        print(f"  {col}: min={df[col].min()}, max={df[col].max()}, unique={df[col].nunique()}")

# Count unique voxel positions
vx = [c for c in voxel_cols if "x" in c.lower()]
vy = [c for c in voxel_cols if "y" in c.lower()]
if vx and vy:
    unique_voxels = df[[vx[0], vy[0]]].drop_duplicates()
    print(f"  Unique voxel positions visited: {len(unique_voxels)}")

# Starting and ending position
if loc_cols:
    print(f"\n  Starting position: ({df[loc_cols[0]].iloc[0]:.2f}, {df[loc_cols[1]].iloc[0]:.2f})" if len(loc_cols) >= 2 else "")
    print(f"  Ending position:   ({df[loc_cols[0]].iloc[-1]:.2f}, {df[loc_cols[1]].iloc[-1]:.2f})" if len(loc_cols) >= 2 else "")

# 9. Trends over time (10 segments)
print(f"\n{'=' * 80}")
print("SECTION 6: TRENDS OVER TIME (10 SEGMENTS)")
print("=" * 80)
trend_metrics = ["efficiency", "resonance", "coherence", "stiffness"]
avail_trend = [m for m in trend_metrics if m in df.columns]
if avail_trend:
    n = len(df)
    seg_size = n // 10
    rows = []
    for i in range(10):
        start = i * seg_size
        end = (i + 1) * seg_size if i < 9 else n
        seg = df.iloc[start:end]
        row = {"segment": f"Seg{i+1} (ticks {start}-{end})"}
        for m in avail_trend:
            row[f"{m}_mean"] = seg[m].mean()
            row[f"{m}_std"] = seg[m].std()
        if "tick" in df.columns:
            row["tick_range"] = f"{seg['tick'].min()}-{seg['tick'].max()}"
        rows.append(row)
    trend_df = pd.DataFrame(rows).set_index("segment")
    pd.set_option("display.max_columns", 20)
    pd.set_option("display.width", 200)
    print(trend_df.round(6).to_string())

# 10. Vehicle distribution
print(f"\n{'=' * 80}")
print("SECTION 7: VEHICLE DISTRIBUTION")
print("=" * 80)
if "vehicle" in df.columns:
    vc = df["vehicle"].value_counts()
    total = len(df)
    for v, count in vc.items():
        print(f"  {v:20s}: {count:8,} ({count/total*100:.1f}%)")
else:
    print("  'vehicle' column not found")

# 11. First and last rows
print(f"\n{'=' * 80}")
print("SECTION 8: FIRST 3 ROWS")
print("=" * 80)
print(df.head(3).to_string())

print(f"\n{'=' * 80}")
print("SECTION 9: LAST 3 ROWS")
print("=" * 80)
print(df.tail(3).to_string())

# 12. Efficiency > 1.0 analysis (resonance amplification)
print(f"\n{'=' * 80}")
print("SECTION 10: RESONANCE AMPLIFICATION (EFFICIENCY > 1.0)")
print("=" * 80)
if "efficiency" in df.columns:
    above_1 = df[df["efficiency"] > 1.0]
    print(f"  Ticks with efficiency > 1.0: {len(above_1)} / {len(df)} ({len(above_1)/len(df)*100:.1f}%)")
    if len(above_1) > 0:
        print(f"  Max efficiency: {above_1['efficiency'].max():.6f}")
        print(f"  Mean efficiency when > 1.0: {above_1['efficiency'].mean():.6f}")

# 13. Coherence stability
print(f"\n{'=' * 80}")
print("SECTION 11: COHERENCE STABILITY")
print("=" * 80)
if "coherence" in df.columns:
    below_07 = df[df["coherence"] < 0.7]
    print(f"  Ticks with coherence < 0.7 (motor locked): {len(below_07)} / {len(df)} ({len(below_07)/len(df)*100:.1f}%)")
    print(f"  Ticks with coherence >= 0.7 (motor unlocked): {len(df) - len(below_07)} / {len(df)} ({(len(df)-len(below_07))/len(df)*100:.1f}%)")
    print(f"  Min coherence: {df['coherence'].min():.6f}")
    print(f"  Mean coherence: {df['coherence'].mean():.6f}")

# 14. Compare runs if multiple exist
all_runs = fi_df["run_id"].unique()
if len(all_runs) > 1:
    print(f"\n{'=' * 80}")
    print("SECTION 12: CROSS-RUN COMPARISON")
    print("=" * 80)
    for rid in all_runs:
        run_files = fi_df[fi_df["run_id"] == rid].sort_values("epoch")
        run_dfs = []
        for _, row in run_files.iterrows():
            try:
                run_dfs.append(pd.read_parquet(row["path"]))
            except:
                pass
        if run_dfs:
            rdf = pd.concat(run_dfs, ignore_index=True)
            print(f"\n  Run: {rid}")
            print(f"    Ticks: {len(rdf):,}")
            for m in ["coherence", "efficiency", "resonance"]:
                if m in rdf.columns:
                    print(f"    {m}: mean={rdf[m].mean():.4f}, std={rdf[m].std():.4f}, min={rdf[m].min():.4f}, max={rdf[m].max():.4f}")
            if "location_x" in rdf.columns and "location_y" in rdf.columns:
                xr = rdf["location_x"].max() - rdf["location_x"].min()
                yr = rdf["location_y"].max() - rdf["location_y"].min()
                print(f"    Movement range: x={xr:.1f}, y={yr:.1f}")

print(f"\n{'=' * 80}")
print("END OF REPORT")
print("=" * 80)
