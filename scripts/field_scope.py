#!/usr/bin/env python3
"""
Field Scope — 2D Top-Down Agent Trajectory Visualizer
======================================================
A static (non-live) visualizer that loads Parquet metrics and renders a
2D scatter plot of the agent's path, colored by Energy Efficiency.

Supports both the original "standard" (viridis) and "resonance" (Harmonic
Scale: Red -> Violet) color modes.

Now reads the `engine_hours` column when available and annotates the plot
with the agent's biological age.

Optimized for Orange Pi / DietPi: uses ax.scatter with alpha=0.6.
"""

import argparse
from pathlib import Path
import sys
from typing import List, Optional, Tuple

import numpy as np
import pandas as pd
import matplotlib.pyplot as plt
from matplotlib.colors import LinearSegmentedColormap, Normalize
from matplotlib.lines import Line2D


DEFAULT_METRICS_DIR = Path(__file__).resolve().parents[1] / "data" / "metrics"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="2D field scope visualizer for agent trajectories."
    )
    parser.add_argument(
        "--mode",
        choices=["standard", "resonance"],
        default="resonance",
        help="Color mapping mode (default: resonance).",
    )
    parser.add_argument(
        "--input",
        type=Path,
        default=DEFAULT_METRICS_DIR,
        help="Parquet file or metrics directory (default: data/metrics).",
    )
    return parser.parse_args()


def load_metrics(path: Path) -> pd.DataFrame:
    if not path.exists():
        raise FileNotFoundError(f"Metrics path not found: {path}")

    if path.is_dir():
        files = sorted(path.glob("*.parquet"), key=lambda p: p.stat().st_mtime)
        if not files:
            raise FileNotFoundError(f"No parquet files found in: {path}")
        frames = [pd.read_parquet(file_path) for file_path in files]
        return pd.concat(frames, ignore_index=True)

    return pd.read_parquet(path)


def pick_position_columns(df: pd.DataFrame) -> Tuple[str, str]:
    """Pick 2D position columns (X and Y only)."""
    location_cols = ("location_x", "location_y")
    voxel_cols = ("voxel_x", "voxel_y")

    if all(col in df.columns for col in location_cols):
        return location_cols
    if all(col in df.columns for col in voxel_cols):
        return voxel_cols

    raise ValueError(
        "Metrics data must include location_x/y or voxel_x/y columns."
    )


def pick_time_column(df: pd.DataFrame) -> Optional[str]:
    if "tick" in df.columns:
        return "tick"
    if "timestamp_ms" in df.columns:
        return "timestamp_ms"
    return None


def build_alpha(count: int) -> np.ndarray:
    if count <= 1:
        return np.array([1.0])
    return np.linspace(0.1, 1.0, count)


def build_resonance_colormap() -> Tuple[LinearSegmentedColormap, Normalize]:
    thresholds = [0.0, 0.6, 0.8, 0.95, 1.05, 1.2, 1.5]
    colors = ["red", "orange", "yellow", "green", "blue", "indigo", "violet"]
    vmax = thresholds[-1]
    epsilon = 1e-6

    points: List[Tuple[float, str]] = [(thresholds[0], colors[0])]
    for idx in range(1, len(thresholds)):
        boundary = thresholds[idx]
        prev_color = colors[idx - 1]
        next_color = colors[idx]
        left = max(thresholds[0], boundary - epsilon)
        points.append((left, prev_color))
        points.append((boundary, next_color))

    normalized = [(value / vmax, color) for value, color in points]
    cmap = LinearSegmentedColormap.from_list("resonance", normalized)
    norm = Normalize(vmin=thresholds[0], vmax=vmax, clip=True)
    return cmap, norm


def resonance_legend_handles() -> List[Line2D]:
    legend_items = [
        ("High Resistance (< 0.60)", "red"),
        ("Friction (0.60 - 0.80)", "orange"),
        ("Transient (0.80 - 0.95)", "yellow"),
        ("Equilibrium (0.95 - 1.05)", "green"),
        ("Coherence (1.05 - 1.20)", "blue"),
        ("Predictive Phase-Locking (1.20 - 1.50)", "indigo"),
        ("Superconductive (> 1.50)", "violet"),
    ]
    return [
        Line2D(
            [0],
            [0],
            marker="o",
            linestyle="",
            markerfacecolor=color,
            markeredgecolor="black",
            markersize=7,
            label=label,
        )
        for label, color in legend_items
    ]


def main() -> int:
    args = parse_args()

    try:
        df = load_metrics(args.input)
    except (FileNotFoundError, ValueError) as exc:
        print(f"Error: {exc}", file=sys.stderr)
        return 1

    if df.empty:
        print("Error: metrics data is empty.", file=sys.stderr)
        return 1

    if "efficiency" not in df.columns:
        print("Error: metrics data missing efficiency column.", file=sys.stderr)
        return 1

    x_col, y_col = pick_position_columns(df)
    time_col = pick_time_column(df)

    if time_col:
        df = df.sort_values(time_col).reset_index(drop=True)

    x = df[x_col].to_numpy()
    y = df[y_col].to_numpy()
    values = df["efficiency"].to_numpy()
    alphas = build_alpha(len(df))

    # Read engine hours if available
    engine_hours_max = None
    if "engine_hours" in df.columns:
        engine_hours_max = float(df["engine_hours"].max())

    # Create 2D figure (no 3D projection)
    fig, ax = plt.subplots(figsize=(10, 8), facecolor='#0a0e27')
    ax.set_facecolor('#0d1117')

    if args.mode == "standard":
        cmap = plt.get_cmap("viridis")
        norm = Normalize(vmin=np.nanmin(values), vmax=np.nanmax(values))
    else:
        cmap, norm = build_resonance_colormap()

    colors = cmap(norm(values))
    colors[:, 3] = alphas

    # 2D scatter with alpha for fast rendering on Orange Pi
    ax.scatter(x, y, c=colors, s=10, alpha=0.6, edgecolors='none')
    ax.set_xlabel("Location X", color='#00ff88', fontsize=11)
    ax.set_ylabel("Location Y", color='#00ff88', fontsize=11)
    ax.tick_params(colors='#557788', labelsize=9)
    ax.grid(True, color='#1a2633', linestyle='-', linewidth=0.5, alpha=0.7)
    for spine in ax.spines.values():
        spine.set_color('#1a2633')

    # Mark latest position
    if len(x) > 0:
        ax.plot(x[-1], y[-1], 'o', color='#00ff88',
                markersize=10, markeredgecolor='white', markeredgewidth=2)

    title_mode = "Standard" if args.mode == "standard" else "Resonance"
    title_text = f"Field Scope Path ({title_mode} Mode) — 2D Top-Down"
    if engine_hours_max is not None:
        title_text += f"\nAGE: {engine_hours_max:.4f} ENGINE HOURS"
    ax.set_title(title_text, color='#00ff88', fontsize=14, pad=15)

    mappable = plt.cm.ScalarMappable(norm=norm, cmap=cmap)
    mappable.set_array(values)
    cbar = fig.colorbar(mappable, ax=ax, shrink=0.8, pad=0.02)
    cbar.set_label("Efficiency (eta)", color='#00ff88')
    cbar.ax.tick_params(colors='#557788')

    if args.mode == "resonance":
        handles = resonance_legend_handles()
        ax.legend(
            handles=handles,
            title="Efficiency State",
            loc="upper left",
            bbox_to_anchor=(1.15, 1.0),
            borderaxespad=0.0,
            facecolor='#0a0e27',
            edgecolor='#1a2633',
            labelcolor='#00ff88',
            title_fontsize=9,
            fontsize=8,
        )

    fig.tight_layout()
    plt.show()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
