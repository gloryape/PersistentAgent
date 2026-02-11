"""
Sanctuary Field Renderer — Reconstruct scalar field from Parquet for visualization.

Reconstructs a 2D field grid from MetricRecord data (Option A). Renders:
- Physical field: phase → hue, resonance → brightness (with synthetic decay)
- Ethical field: efficiency → HarmonicAscension colormap

Limitation: Parquet stores interaction state, not current voxel amplitude ρ.
Synthetic exponential decay approximates field fading (ENTROPY_DECAY).
"""

from __future__ import annotations

import math
from dataclasses import dataclass, field
from typing import Optional, Tuple, Any

import numpy as np
import pandas as pd

# Constants (match Rust sanctuary.rs where applicable)
VOXEL_SIZE_PX = 10
VOID_COLOR = (0.0, 0.0, 0.0)
AGENT_MARKER_COLOR = (1.0, 1.0, 1.0)
AGENT_MARKER_SIZE = 8
AGENT_ARROW_LENGTH = 6
ENTROPY_DECAY = 0.05  # Must match sanctuary.rs
MIN_VIEWPORT_WIDTH = 60
MIN_VIEWPORT_HEIGHT = 45
MAX_GRID_WIDTH = 400
MAX_GRID_HEIGHT = 300

# Ticks per second (90 Hz)
TICKS_PER_SECOND = 90.0


@dataclass
class FieldGrid:
    """Structured field data for rendering. Bounds and arrays in voxel space."""
    x_min: int
    x_max: int
    y_min: int
    y_max: int
    width: int   # x_max - x_min + 1
    height: int  # y_max - y_min + 1
    phase_grid: np.ndarray      # (height, width), float
    resonance_grid: np.ndarray  # (height, width), float
    efficiency_grid: np.ndarray # (height, width), float
    tick_grid: np.ndarray       # (height, width), float (tick of last interaction)
    current_tick: float
    agent_position: Optional[Tuple[int, int]]  # (voxel_x, voxel_y) for grid indexing
    agent_phase: Optional[float]  # radians, for direction arrow
    agent_coherence: Optional[float]  # 0.0–1.0 metabolic coherence (motor unlock %)
    max_tick: float  # for incremental caching
    _voxel_df: Optional[pd.DataFrame] = field(default=None, repr=False)  # for incremental merge


def _ensure_columns(df: pd.DataFrame) -> bool:
    """Return True if df has required columns for field reconstruction."""
    required = {'voxel_x', 'voxel_y', 'phase', 'resonance', 'efficiency', 'tick'}
    return required.issubset(set(df.columns))


def reconstruct_field_from_parquet(
    df: pd.DataFrame,
    harmonic_cmap: Any = None,
    cached_grid: Optional[FieldGrid] = None,
) -> FieldGrid:
    """
    Build a FieldGrid from Parquet MetricRecords.
    Groups by (voxel_x, voxel_y), takes most recent record per voxel.
    Optionally merges new records into cached_grid for incremental updates.
    """
    if df is None or df.empty or not _ensure_columns(df):
        if cached_grid is not None:
            return cached_grid
        # Return minimal empty grid
        return FieldGrid(
            x_min=0, x_max=MIN_VIEWPORT_WIDTH - 1,
            y_min=0, y_max=MIN_VIEWPORT_HEIGHT - 1,
            width=MIN_VIEWPORT_WIDTH, height=MIN_VIEWPORT_HEIGHT,
            phase_grid=np.zeros((MIN_VIEWPORT_HEIGHT, MIN_VIEWPORT_WIDTH), dtype=np.float64),
            resonance_grid=np.zeros((MIN_VIEWPORT_HEIGHT, MIN_VIEWPORT_WIDTH), dtype=np.float64),
            efficiency_grid=np.zeros((MIN_VIEWPORT_HEIGHT, MIN_VIEWPORT_WIDTH), dtype=np.float64),
            tick_grid=np.full((MIN_VIEWPORT_HEIGHT, MIN_VIEWPORT_WIDTH), -1e9, dtype=np.float64),
            current_tick=0.0,
            agent_position=None,
            agent_phase=None,
            agent_coherence=None,
            max_tick=0.0,
        )

    df_sorted = df.sort_values('tick')
    max_tick = float(df_sorted['tick'].iloc[-1])

    # Incremental path: merge new records into cached voxel dataframe
    if cached_grid is not None and cached_grid._voxel_df is not None:
        new_records = df_sorted[df_sorted['tick'] > cached_grid.max_tick]
        if new_records.empty:
            # Update current_tick and agent from full df (latest row)
            last = df_sorted.iloc[-1]
            agent_vx = int(last['voxel_x']) if 'voxel_x' in last else None
            agent_vy = int(last['voxel_y']) if 'voxel_y' in last else None
            agent_ph = float(last['phase']) if 'phase' in last else None
            agent_coh = float(last['coherence']) if 'coherence' in last.index else None
            agent_pos = (agent_vx, agent_vy) if agent_vx is not None and agent_vy is not None else None
            return FieldGrid(
                x_min=cached_grid.x_min, x_max=cached_grid.x_max,
                y_min=cached_grid.y_min, y_max=cached_grid.y_max,
                width=cached_grid.width, height=cached_grid.height,
                phase_grid=cached_grid.phase_grid.copy(),
                resonance_grid=cached_grid.resonance_grid.copy(),
                efficiency_grid=cached_grid.efficiency_grid.copy(),
                tick_grid=cached_grid.tick_grid.copy(),
                current_tick=max_tick,
                agent_position=agent_pos,
                agent_phase=agent_ph,
                agent_coherence=agent_coh,
                max_tick=cached_grid.max_tick,
                _voxel_df=cached_grid._voxel_df,
            )

        # Merge: combine cached voxel data with new records, then take last per voxel
        new_latest = new_records.groupby(['voxel_x', 'voxel_y']).agg({
            'phase': 'last',
            'resonance': 'last',
            'efficiency': 'last',
            'tick': 'last',
            'location_x': 'last',
            'location_y': 'last',
        }).reset_index()
        combined = pd.concat([cached_grid._voxel_df, new_latest], ignore_index=True)
        latest = combined.sort_values('tick').groupby(['voxel_x', 'voxel_y']).agg({
            'phase': 'last', 'resonance': 'last', 'efficiency': 'last', 'tick': 'last',
            'location_x': 'last', 'location_y': 'last',
        }).reset_index()
    else:
        # Full rebuild
        latest = df_sorted.groupby(['voxel_x', 'voxel_y']).agg({
            'phase': 'last',
            'resonance': 'last',
            'efficiency': 'last',
            'tick': 'last',
            'location_x': 'last',
            'location_y': 'last',
        }).reset_index()

    if latest.empty:
        if cached_grid is not None:
            return cached_grid
        return FieldGrid(
            x_min=0, x_max=MIN_VIEWPORT_WIDTH - 1,
            y_min=0, y_max=MIN_VIEWPORT_HEIGHT - 1,
            width=MIN_VIEWPORT_WIDTH, height=MIN_VIEWPORT_HEIGHT,
            phase_grid=np.zeros((MIN_VIEWPORT_HEIGHT, MIN_VIEWPORT_WIDTH), dtype=np.float64),
            resonance_grid=np.zeros((MIN_VIEWPORT_HEIGHT, MIN_VIEWPORT_WIDTH), dtype=np.float64),
            efficiency_grid=np.zeros((MIN_VIEWPORT_HEIGHT, MIN_VIEWPORT_WIDTH), dtype=np.float64),
            tick_grid=np.full((MIN_VIEWPORT_HEIGHT, MIN_VIEWPORT_WIDTH), -1e9, dtype=np.float64),
            current_tick=max_tick,
            agent_position=None,
            agent_phase=None,
            agent_coherence=None,
            max_tick=max_tick,
            _voxel_df=latest,
        )

    x_min = int(latest['voxel_x'].min())
    x_max = int(latest['voxel_x'].max())
    y_min = int(latest['voxel_y'].min())
    y_max = int(latest['voxel_y'].max())

    # Agent position, phase, and coherence from latest record in full df
    last_row = df_sorted.iloc[-1]
    agent_vx = int(last_row['voxel_x'])
    agent_vy = int(last_row['voxel_y'])
    agent_phase = float(last_row['phase'])
    agent_coherence = float(last_row['coherence']) if 'coherence' in last_row.index else None

    # Bounds: clamp to max size (agent-centered), expand to min size
    width_raw = x_max - x_min + 1
    height_raw = y_max - y_min + 1

    if width_raw > MAX_GRID_WIDTH or height_raw > MAX_GRID_HEIGHT:
        half_w = MAX_GRID_WIDTH // 2
        half_h = MAX_GRID_HEIGHT // 2
        x_min = max(x_min, agent_vx - half_w)
        x_max = min(x_max, agent_vx + half_w)
        y_min = max(y_min, agent_vy - half_h)
        y_max = min(y_max, agent_vy + half_h)
        width_raw = x_max - x_min + 1
        height_raw = y_max - y_min + 1

    if width_raw < MIN_VIEWPORT_WIDTH:
        pad = (MIN_VIEWPORT_WIDTH - width_raw) // 2
        x_min -= pad
        x_max += (MIN_VIEWPORT_WIDTH - width_raw - pad)
    if height_raw < MIN_VIEWPORT_HEIGHT:
        pad = (MIN_VIEWPORT_HEIGHT - height_raw) // 2
        y_min -= pad
        y_max += (MIN_VIEWPORT_HEIGHT - height_raw - pad)

    width = x_max - x_min + 1
    height = y_max - y_min + 1

    phase_grid = np.zeros((height, width), dtype=np.float64)
    resonance_grid = np.zeros((height, width), dtype=np.float64)
    efficiency_grid = np.zeros((height, width), dtype=np.float64)
    tick_grid = np.full((height, width), -1e9, dtype=np.float64)

    for _, row in latest.iterrows():
        vx, vy = int(row['voxel_x']), int(row['voxel_y'])
        gx = vx - x_min
        gy = vy - y_min
        if 0 <= gx < width and 0 <= gy < height:
            phase_grid[gy, gx] = float(row['phase'])
            resonance_grid[gy, gx] = float(row['resonance'])
            efficiency_grid[gy, gx] = float(row['efficiency'])
            tick_grid[gy, gx] = float(row['tick'])

    # Agent position in voxel space (dashboard computes ax_x = agent_position[0] - x_min)
    agent_position = (agent_vx, agent_vy)

    return FieldGrid(
        x_min=x_min, x_max=x_max, y_min=y_min, y_max=y_max,
        width=width, height=height,
        phase_grid=phase_grid,
        resonance_grid=resonance_grid,
        efficiency_grid=efficiency_grid,
        tick_grid=tick_grid,
        current_tick=max_tick,
        agent_position=agent_position,
        agent_phase=agent_phase,
        agent_coherence=agent_coherence,
        max_tick=max_tick,
        _voxel_df=latest,
    )


def render_physical_field(
    grid: FieldGrid,
    harmonic_cmap: Any,
) -> Tuple[np.ndarray, np.ndarray]:
    """
    Render physical field (phase + resonance) and ethical field (efficiency).
    Returns (physical_rgb, ethical_rgb) each (height, width, 3) in [0,1].
    Unvisited voxels are black. Synthetic decay applied by tick age.
    """
    import matplotlib.colors as mcolors

    h, w = grid.height, grid.width
    physical_rgb = np.zeros((h, w, 3), dtype=np.float64)
    ethical_rgb = np.zeros((h, w, 3), dtype=np.float64)

    # Mask of visited voxels (tick was set)
    visited = grid.tick_grid >= 0

    if not np.any(visited):
        return physical_rgb, ethical_rgb

    # Synthetic decay: brightness *= exp(-(current_tick - record_tick) * ENTROPY_DECAY / 90)
    decay_factor = np.exp(
        -np.maximum(0.0, grid.current_tick - grid.tick_grid) * ENTROPY_DECAY / TICKS_PER_SECOND
    )
    decay_factor = np.where(visited, decay_factor, 0.0)

    # Physical: hue = phase / (2π), saturation = 0.85, value = resonance * decay
    hue = (np.mod(grid.phase_grid, 2.0 * math.pi) / (2.0 * math.pi)).astype(np.float64)
    sat = np.full_like(hue, 0.85)
    resonance_clip = np.clip(grid.resonance_grid, 0.0, 1.0)
    value = np.clip(resonance_clip * decay_factor, 0.0, 1.0)
    hsv = np.stack([hue, sat, value], axis=-1)
    rgb_physical = mcolors.hsv_to_rgb(hsv)
    physical_rgb[visited] = rgb_physical[visited]

    # Ethical: HarmonicAscension colormap on efficiency with decay
    decayed_eff = grid.efficiency_grid * decay_factor
    decayed_eff = np.where(visited, decayed_eff, 0.0)
    # Normalize into colormap range; unvisited stays 0 (black)
    norm = harmonic_cmap.norm
    eff_norm = np.clip(norm(decayed_eff), 0.0, 1.0)
    ethical_colors = harmonic_cmap.cmap(eff_norm)
    ethical_rgb[visited] = ethical_colors[visited, :3]

    return physical_rgb, ethical_rgb
