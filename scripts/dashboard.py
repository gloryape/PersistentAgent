#!/usr/bin/env python3
"""
Sanctuary Dashboard - Mission Control (2D Engine Hours Edition)
===============================================================
A high-performance 2D "Flatland" visualization system for monitoring the
Quaternity Organism's interaction with the Sanctuary field.

Architecture:
- 2D Top-Down scatter plot (X vs Y) — eliminates 40-60% GPU overhead vs 3D
- Color encodes the Z-axis of "Quality" (Energy Efficiency via Harmonic Scale)
- Engine Hours display: treats agent like a machine accumulating experience
- History Scrubber: playback slider to review past states by engine hour

Optimized for Orange Pi / DietPi with minimal computational footprint.
"""

import tkinter as tk
from tkinter import ttk, filedialog, messagebox
from pathlib import Path
import subprocess
import sys
import threading
import time
import os
from typing import Optional, Tuple, List

import numpy as np
import pandas as pd
import matplotlib
matplotlib.use('TkAgg')
import matplotlib.pyplot as plt
from matplotlib.backends.backend_tkagg import FigureCanvasTkAgg
from matplotlib.figure import Figure
from matplotlib.colors import LinearSegmentedColormap, Normalize

# Paths (dashboard lives in quaternity_organism/scripts/)
SCRIPT_DIR = Path(__file__).resolve().parent
ORGANISM_DIR = SCRIPT_DIR.parent
DEFAULT_METRICS_DIR = (ORGANISM_DIR / "data" / "metrics").resolve()
SIGNAL_GENERATOR_PATH = SCRIPT_DIR / "signal_generator.py"
OPTIC_NERVE_PATH = SCRIPT_DIR / "optic_nerve.py"
# Prefer running the release binary directly to avoid Cargo's artifact-dir lock
RELEASE_BINARY = ORGANISM_DIR / "target" / "release" / ("quaternity-organism.exe" if sys.platform == "win32" else "quaternity-organism")
# Error log: all preflight and process errors are appended here for debugging
LOG_DIR = ORGANISM_DIR / "logs"
DASHBOARD_ERROR_LOG = LOG_DIR / "dashboard_errors.log"


def _log_error(kind: str, title: str, message: str, details: Optional[str] = None) -> Path:
    """Append an error to the dashboard error log. Returns the log file path."""
    try:
        LOG_DIR.mkdir(parents=True, exist_ok=True)
        with open(DASHBOARD_ERROR_LOG, "a", encoding="utf-8") as f:
            from datetime import datetime
            ts = datetime.utcnow().strftime("%Y-%m-%d %H:%M:%S UTC")
            f.write(f"\n{'='*60}\n[{ts}] {kind}\n{title}\n{'-'*40}\n{message}\n")
            if details:
                f.write(f"\n--- details ---\n{details}\n")
            f.write(f"{'='*60}\n")
    except Exception as e:
        print(f"Could not write error log: {e}")
    return DASHBOARD_ERROR_LOG


def _env_with_cargo_path():
    """Return env dict with Cargo bin on PATH so subprocess finds cargo."""
    env = os.environ.copy()
    # Rust/cargo typically on Windows: %USERPROFILE%\.cargo\bin
    user_home = os.environ.get("USERPROFILE") or os.environ.get("HOME") or ""
    cargo_bin = os.path.join(user_home, ".cargo", "bin")
    if os.path.isdir(cargo_bin):
        path_sep = ";" if sys.platform == "win32" else ":"
        env["PATH"] = cargo_bin + path_sep + env.get("PATH", "")
    return env


def _find_cargo(run_env):
    """Return path to cargo.exe (or 'cargo') if found in run_env PATH, else None."""
    path = run_env.get("PATH", "")
    path_sep = ";" if sys.platform == "win32" else ":"
    for part in path.split(path_sep):
        part = part.strip()
        if not part:
            continue
        cargo_exe = os.path.join(part, "cargo.exe" if sys.platform == "win32" else "cargo")
        if os.path.isfile(cargo_exe):
            return cargo_exe
    return None


def _preflight_checks(init_mode: bool, use_generator: bool, stimulus_file: Optional[str]) -> Optional[str]:
    """
    Run before starting the entity. Returns None if OK, or an error message to show the user.
    """
    run_env = _env_with_cargo_path()
    cargo_path = _find_cargo(run_env)
    if not cargo_path:
        return (
            "Cargo not found.\n\n"
            "Install the Rust toolchain, then restart the dashboard.\n\n"
            "Windows (PowerShell, run as user):\n"
            "  winget install Rustlang.Rustup\n"
            "  # Close and reopen your terminal/IDE so PATH is updated.\n\n"
            "Or download and run: https://rustup.rs\n"
            "Choose '1) Proceed with installation' and ensure it adds Cargo to PATH."
        )
    if not (ORGANISM_DIR / "Cargo.toml").exists():
        return (
            f"Not a Rust project directory.\n\n"
            f"Expected Cargo.toml in:\n{ORGANISM_DIR}"
        )
    if use_generator and not SIGNAL_GENERATOR_PATH.exists():
        return (
            f"Signal generator script not found.\n\n"
            f"Expected: {SIGNAL_GENERATOR_PATH}"
        )
    if stimulus_file and not os.path.isfile(stimulus_file):
        return (
            f"Video file not found.\n\n"
            f"Path: {stimulus_file}"
        )
    if not use_generator and stimulus_file:
        # Video file mode — need optic_nerve.py and cv2
        if not OPTIC_NERVE_PATH.exists():
            return (
                f"Optic nerve script not found.\n\n"
                f"Expected: {OPTIC_NERVE_PATH}"
            )
        try:
            r = subprocess.run(
                [sys.executable, "-c", "import cv2; print('ok')"],
                capture_output=True,
                timeout=10,
                cwd=ORGANISM_DIR,
                env=run_env,
            )
            if r.returncode != 0:
                return (
                    "OpenCV not installed (required for video input).\n\n"
                    "Install with: pip install opencv-python-headless"
                )
        except (subprocess.TimeoutExpired, FileNotFoundError) as e:
            return f"Could not verify OpenCV: {e}"
    if init_mode and use_generator:
        # Quick check that Python can run the generator (avoid starting pipeline to fail)
        try:
            r = subprocess.run(
                [sys.executable, "-c", "import numpy; print('ok')"],
                capture_output=True,
                timeout=5,
                cwd=ORGANISM_DIR,
                env=run_env,
            )
            if r.returncode != 0:
                return (
                    "Python environment issue.\n\n"
                    "The dashboard needs Python with numpy. Install with: pip install numpy"
                )
        except (subprocess.TimeoutExpired, FileNotFoundError) as e:
            return f"Could not verify Python: {e}"
    return None


def _explain_exit_code(exit_code: int, output_lines: List[str]) -> Tuple[str, str]:
    """Turn exit code and captured output into a short title and actionable message."""
    output_text = "\n".join(output_lines[-25:]) if output_lines else "(no output)"
    output_lower = output_text.lower()
    title = f"Entity exited ({exit_code})"
    if exit_code in (255, 101):
        if "cargo" in output_lower and ("not recognized" in output_lower or "not found" in output_lower):
            return title, (
                "Cargo was not found when running the command.\n\n"
                "• Install Rust from https://rustup.rs and add to PATH, or\n"
                "• Restart the dashboard after installing Rust.\n\n"
                "Last output:\n" + output_text
            )
        if "link.exe" in output_lower or "linker" in output_lower and "not found" in output_lower:
            return title, (
                "Rust needs the MSVC linker (link.exe) to build on Windows.\n\n"
                "Install one of:\n"
                "• Build Tools for Visual Studio: https://visualstudio.microsoft.com/visual-cpp-build-tools/\n"
                "  — Select workload 'Desktop development with C++'\n"
                "• Or full Visual Studio with C++ workload\n\n"
                "Then restart the dashboard and try again.\n\n"
                "Last output:\n" + output_text
            )
        if "error" in output_lower and "compiling" in output_lower:
            return title, (
                "Rust build failed (compile error).\n\n"
                "Fix the errors shown below, then try again.\n\n"
                "Last output:\n" + output_text
            )
        if "panic" in output_lower or "thread 'main' panicked" in output_lower:
            return title, (
                "The entity process panicked.\n\n"
                "Check the output below for the panic message and fix the cause.\n\n"
                "Last output:\n" + output_text
            )
        if "no state file" in output_lower or "entropy has won" in output_lower:
            return title, (
                "Resume failed: no saved state found.\n\n"
                "Use 'Initialize Entity' first to create a run, then Stop; after that you can Resume.\n\n"
                "Last output:\n" + output_text
            )
    # Generic
    return title, (
        "The entity process exited unexpectedly.\n\n"
        "Check the output below for errors. Common fixes:\n"
        "• Install Rust (rustup.rs) and ensure cargo is on PATH\n"
        "• Run from a terminal where 'cargo' works, then start the dashboard from that same environment\n\n"
        "Last output:\n" + output_text
    )


class HarmonicColormap:
    """
    Custom colormap for ethical emergence visualization.
    Maps Energy Efficiency to the Harmonic Scale (Red -> Violet).
    
    Thresholds:
    - 0.0 - 0.6: Red (Survival/Root)
    - 0.6 - 0.8: Orange (Friction)
    - 0.8 - 0.95: Yellow (Effort)
    - 0.95 - 1.05: Green (Equilibrium/Heart)
    - 1.05 - 1.2: Blue (Coherence)
    - 1.2 - 1.5: Indigo (Prediction)
    - 1.5+: Violet/White (Super-Conductance)
    """
    
    def __init__(self):
        self.thresholds = [0.0, 0.6, 0.8, 0.95, 1.05, 1.2, 1.5]
        self.colors = ['red', 'orange', 'yellow', 'green', 'blue', 'indigo', 'violet']
        self.vmax = 1.8  # Extended range for super-conductance
        self.cmap, self.norm = self._build_colormap()
        
    def _build_colormap(self) -> Tuple[LinearSegmentedColormap, Normalize]:
        """Build the segmented colormap with sharp transitions."""
        epsilon = 1e-6
        points: List[Tuple[float, str]] = [(self.thresholds[0], self.colors[0])]
        
        for idx in range(1, len(self.thresholds)):
            boundary = self.thresholds[idx]
            prev_color = self.colors[idx - 1]
            next_color = self.colors[idx]
            left = max(self.thresholds[0], boundary - epsilon)
            points.append((left, prev_color))
            points.append((boundary, next_color))
        
        # Add super-conductance (violet to white transition)
        points.append((self.vmax - epsilon, 'violet'))
        points.append((self.vmax, 'white'))
        
        normalized = [(value / self.vmax, color) for value, color in points]
        cmap = LinearSegmentedColormap.from_list("HarmonicAscension", normalized)
        norm = Normalize(vmin=self.thresholds[0], vmax=self.vmax, clip=True)
        return cmap, norm
    
    def get_legend_labels(self) -> List[Tuple[str, str]]:
        """Return legend labels with colors."""
        return [
            ("Survival/Root (< 0.60)", "red"),
            ("Friction (0.60 - 0.80)", "orange"),
            ("Effort (0.80 - 0.95)", "yellow"),
            ("Equilibrium (0.95 - 1.05)", "green"),
            ("Coherence (1.05 - 1.20)", "blue"),
            ("Prediction (1.20 - 1.50)", "indigo"),
            ("Super-Conductance (> 1.50)", "violet"),
        ]


class DataReader:
    """
    Efficient parquet file reader with tail length and engine-hour filtering.
    Only reads the latest necessary rows to prevent memory bloat.
    """
    
    def __init__(self, metrics_dir: Path, tail_length: int = 1000):
        self.metrics_dir = metrics_dir
        self.tail_length = tail_length
        self.last_file = None
        self.last_mtime = 0
        self._full_df: Optional[pd.DataFrame] = None
        
    @staticmethod
    def _run_prefix_from_parquet_path(path: Path) -> str:
        """Extract run prefix from filename like run_<uuid>_epoch_000001.parquet."""
        stem = path.stem
        if "_epoch_" in stem:
            return stem.split("_epoch_")[0]
        return stem

    def read_latest_data(self) -> Optional[pd.DataFrame]:
        """Read the latest data from parquet files (current run only in live mode)."""
        if not self.metrics_dir.exists():
            return None

        parquet_files = sorted(
            self.metrics_dir.glob("*.parquet"),
            key=lambda p: p.stat().st_mtime
        )

        if not parquet_files:
            return None

        latest_file = parquet_files[-1]
        current_mtime = latest_file.stat().st_mtime
        self.last_file = latest_file
        self.last_mtime = current_mtime

        # Only read files from the same run as the most recently modified file.
        # Otherwise an old run with higher tick counts dominates tail() and live never updates.
        run_prefix = self._run_prefix_from_parquet_path(latest_file)
        same_run = [p for p in parquet_files if self._run_prefix_from_parquet_path(p) == run_prefix]
        same_run = sorted(same_run, key=lambda p: p.stat().st_mtime)
        num_tail = min(30, len(same_run))
        files_to_read = same_run[-num_tail:] if num_tail > 0 else same_run

        frames = []
        for file_path in files_to_read:
            try:
                df = pd.read_parquet(file_path)
                frames.append(df)
            except Exception as e:
                print(f"Error reading {file_path}: {e}")
                continue

        if not frames:
            return None

        full_df = pd.concat(frames, ignore_index=True)
        if 'tick' in full_df.columns:
            full_df = full_df.sort_values('tick')

        self._full_df = full_df
        return full_df.tail(self.tail_length)

    def read_all_data(self) -> Optional[pd.DataFrame]:
        """Read all available parquet data (for history mode)."""
        if not self.metrics_dir.exists():
            return None

        parquet_files = sorted(
            self.metrics_dir.glob("*.parquet"),
            key=lambda p: p.stat().st_mtime
        )

        if not parquet_files:
            return None

        frames = []
        for file_path in parquet_files:
            try:
                df = pd.read_parquet(file_path)
                frames.append(df)
            except Exception as e:
                print(f"Error reading {file_path}: {e}")
                continue

        if not frames:
            return None

        full_df = pd.concat(frames, ignore_index=True)
        if 'tick' in full_df.columns:
            full_df = full_df.sort_values('tick')

        self._full_df = full_df
        return full_df

    def get_data_at_engine_hour(self, target_hour: float, window_hours: float = 0.01) -> Optional[pd.DataFrame]:
        """
        Get data around a specific engine hour for history scrubbing.
        
        Args:
            target_hour: The engine hour to seek to.
            window_hours: Time window around target (default ~36 seconds).
        
        Returns:
            DataFrame filtered to the requested time window.
        """
        if self._full_df is None:
            self.read_all_data()

        if self._full_df is None or self._full_df.empty:
            return None

        if 'engine_hours' not in self._full_df.columns:
            # Fallback: return tail of full data
            return self._full_df.tail(self.tail_length)

        # Filter to window around target engine hour
        half_window = window_hours / 2.0
        lower = target_hour - half_window
        upper = target_hour + half_window
        mask = (self._full_df['engine_hours'] >= lower) & (self._full_df['engine_hours'] <= upper)
        filtered = self._full_df[mask]

        if filtered.empty:
            # Find closest data point and grab surrounding window
            idx = (self._full_df['engine_hours'] - target_hour).abs().idxmin()
            center = max(0, idx - self.tail_length // 2)
            end = min(len(self._full_df), center + self.tail_length)
            return self._full_df.iloc[center:end]

        return filtered.tail(self.tail_length)

    def get_max_engine_hours(self) -> float:
        """Get the maximum engine hours in the dataset."""
        if self._full_df is not None and 'engine_hours' in self._full_df.columns:
            return float(self._full_df['engine_hours'].max())
        return 0.0

    def set_tail_length(self, length: int):
        """Update the tail length."""
        self.tail_length = length
        # Invalidate cache so the next read picks up data with the new tail length
        self.last_mtime = 0


class EntityDashboard:
    """
    Main dashboard application — 2D "Flatland" Mode.
    
    Architecture:
    - Left plot: Scientific 2D scatter (Viridis colormap)
    - Right plot: Harmonic 2D scatter (Red→Violet ethical emergence)
    - Engine Hours display: prominent AGE readout
    - History Scrubber: slider to review past states
    """
    
    def __init__(self, root: tk.Tk):
        self.root = root
        self.root.title("Entity Control — 2D Engine Hours")
        self.root.geometry("1600x900")
        
        # Apply dark theme
        self._apply_dark_theme()
        
        # State
        self.simulation_process: Optional[subprocess.Popen] = None
        self.stimulus_file: Optional[str] = None
        self.stimulus_source_var = tk.StringVar(value="file")  # "file" | "generator"
        self.generator_mode_var = tk.StringVar(value="Void")   # "Void" | "Coherence"
        self.is_running = False
        self._sim_output_lines: List[str] = []  # capture output to show on failure
        self.tail_length = 1000
        self.live_mode = True  # True = live updates, False = history scrub
        self.history_hour = 0.0  # Current position of history scrubber
        
        # Data reader
        self.data_reader = DataReader(DEFAULT_METRICS_DIR, self.tail_length)
        
        # Cache for last valid data (to avoid flickering when file is temporarily unavailable)
        self.last_valid_df: Optional[pd.DataFrame] = None
        
        # Clear any stale cache on startup to prevent showing old data
        self.data_reader.last_mtime = 0
        self.data_reader.last_file = None
        
        # Colormap
        self.harmonic_cmap = HarmonicColormap()
        
        # Create UI
        self._create_ui()
        
        # Polling-based update (avoids FuncAnimation event_source None crash on stop/iconify)
        self._update_job = None
        self._start_animation()
        
    def _apply_dark_theme(self):
        """Apply cyberpunk/lab dark theme — radar/sonar aesthetic."""
        style = ttk.Style()
        style.theme_use('clam')
        
        # Colors — deep space with neon accents
        self.bg_color = '#0a0e27'    # Deep space blue
        self.fg_color = '#00ff88'    # Neon green
        self.accent_color = '#00ccff'  # Cyan accent
        button_color = '#1a1f3a'
        
        self.root.configure(bg=self.bg_color)
        
        style.configure('TFrame', background=self.bg_color)
        style.configure('TLabel', background=self.bg_color, foreground=self.fg_color, 
                       font=('Courier', 10))
        style.configure('Title.TLabel', background=self.bg_color, foreground=self.fg_color,
                       font=('Courier', 20, 'bold'))
        style.configure('EngineHours.TLabel', background=self.bg_color, 
                       foreground=self.accent_color, font=('Courier', 16, 'bold'))
        style.configure('TButton', background=button_color, foreground=self.fg_color,
                       font=('Courier', 10, 'bold'))
        style.map('TButton', background=[('active', '#2a3f5a')])
        style.configure('TScale', background=self.bg_color)
        style.configure('TCheckbutton', background=self.bg_color, foreground=self.fg_color,
                       font=('Courier', 10))
        
    def _create_ui(self):
        """Create the main UI layout with 2D plots and controls."""
        # Main container
        main_frame = ttk.Frame(self.root)
        main_frame.pack(fill=tk.BOTH, expand=True, padx=10, pady=10)
        
        # === TOP BAR: Title + Engine Hours ===
        top_bar = ttk.Frame(main_frame)
        top_bar.pack(fill=tk.X, pady=(0, 5))
        
        title_label = ttk.Label(
            top_bar, 
            text="ENTITY CONTROL",
            style='Title.TLabel'
        )
        title_label.pack(side=tk.LEFT)
        
        # Engine Hours display (prominent)
        self.engine_hours_label = ttk.Label(
            top_bar,
            text="AGE: 0.0000 HOURS",
            style='EngineHours.TLabel'
        )
        self.engine_hours_label.pack(side=tk.RIGHT, padx=20)
        
        # Mode indicator
        self.mode_label = ttk.Label(
            top_bar,
            text="[LIVE]",
            foreground='#00ff00',
            font=('Courier', 12, 'bold')
        )
        self.mode_label.pack(side=tk.RIGHT, padx=10)
        
        # === VISUALIZATION AREA: Field visualization (physical + ethical) ===
        viz_frame = ttk.Frame(main_frame)
        viz_frame.pack(fill=tk.BOTH, expand=True)
        
        # Create matplotlib figure with 1x2 subplots for field views
        self.fig = Figure(figsize=(16, 7), facecolor=self.bg_color)
        
        # Create grid: 2 main panels + 2 small direction indicators
        gs = self.fig.add_gridspec(1, 4, width_ratios=[20, 1, 20, 1], wspace=0.15)
        
        # Left: Physical field (phase + resonance)
        self.ax_physical = self.fig.add_subplot(gs[0, 0], facecolor='#000000')
        self._style_field_axis(self.ax_physical, 'Physical Field (Memory Resonance)')
        
        # Direction indicator for physical field
        self.ax_dir_phys = self.fig.add_subplot(gs[0, 1], facecolor='#1a1a1a')
        self._style_direction_axis(self.ax_dir_phys, 'Dir')
        
        # Right: Efficiency field (thermodynamic efficiency colormap)
        self.ax_ethical = self.fig.add_subplot(gs[0, 2], facecolor='#000000')
        self._style_field_axis(self.ax_ethical, 'Efficiency Field')
        
        # Direction indicator for efficiency field
        self.ax_dir_eth = self.fig.add_subplot(gs[0, 3], facecolor='#1a1a1a')
        self._style_direction_axis(self.ax_dir_eth, 'Dir')
        
        # Efficiency colorbar for right panel
        from matplotlib.cm import ScalarMappable
        sm = ScalarMappable(cmap=self.harmonic_cmap.cmap, norm=self.harmonic_cmap.norm)
        self.efficiency_colorbar = self.fig.colorbar(
            sm, ax=self.ax_ethical,
            label='Efficiency',
            orientation='vertical',
            fraction=0.046,
            pad=0.04
        )
        self.efficiency_colorbar.ax.yaxis.label.set_color(self.fg_color)
        self.efficiency_colorbar.ax.tick_params(colors=self.fg_color, labelsize=8)
        
        self._fig_layout()
        
        # Embed in tkinter
        self.canvas = FigureCanvasTkAgg(self.fig, master=viz_frame)
        self.canvas.draw()
        self.canvas.get_tk_widget().pack(fill=tk.BOTH, expand=True)
        
        # === HISTORY SCRUBBER BAR ===
        scrubber_frame = ttk.Frame(main_frame)
        scrubber_frame.pack(fill=tk.X, pady=(5, 5))
        
        # Live mode checkbox
        self.live_var = tk.BooleanVar(value=True)
        self.live_checkbox = tk.Checkbutton(
            scrubber_frame,
            text="LIVE",
            variable=self.live_var,
            command=self._toggle_live_mode,
            bg=self.bg_color,
            fg=self.fg_color,
            selectcolor='#1a1f3a',
            activebackground=self.bg_color,
            activeforeground=self.fg_color,
            font=('Courier', 10, 'bold')
        )
        self.live_checkbox.pack(side=tk.LEFT, padx=(0, 10))
        
        ttk.Label(scrubber_frame, text="Engine Hour:").pack(side=tk.LEFT)
        
        self.history_var = tk.DoubleVar(value=0.0)
        self.history_slider = ttk.Scale(
            scrubber_frame,
            from_=0.0,
            to=1.0,
            orient=tk.HORIZONTAL,
            variable=self.history_var,
            command=self._on_history_scrub,
            length=800
        )
        self.history_slider.pack(side=tk.LEFT, fill=tk.X, expand=True, padx=5)
        self.history_slider.configure(state='disabled')  # Disabled in live mode
        
        self.history_value_label = ttk.Label(scrubber_frame, text="0.0000 h")
        self.history_value_label.pack(side=tk.LEFT, padx=5)
        
        # === CONTROL PANEL ===
        control_frame = ttk.Frame(main_frame)
        control_frame.pack(fill=tk.X, pady=(5, 0))
        
        # Left controls
        left_controls = ttk.Frame(control_frame)
        left_controls.pack(side=tk.LEFT, fill=tk.X, expand=True)
        
        # Input source
        input_frame = ttk.Frame(left_controls)
        input_frame.pack(side=tk.LEFT, padx=5)
        
        ttk.Label(input_frame, text="Input Source:").pack(side=tk.LEFT)
        ttk.Radiobutton(
            input_frame,
            text="External File (Optic Nerve)",
            variable=self.stimulus_source_var,
            value="file",
            command=self._on_input_source_change
        ).pack(side=tk.LEFT, padx=(10, 5))
        ttk.Radiobutton(
            input_frame,
            text="Signal Generator",
            variable=self.stimulus_source_var,
            value="generator",
            command=self._on_input_source_change
        ).pack(side=tk.LEFT, padx=5)
        
        # External: Browse + label
        self.load_file_btn = ttk.Button(
            input_frame,
            text="Browse",
            command=self._load_file
        )
        self.load_file_btn.pack(side=tk.LEFT, padx=(10, 2))
        self.file_label = ttk.Label(input_frame, text="None", foreground='#ff6600')
        self.file_label.pack(side=tk.LEFT, padx=2)
        
        # Generator: mode dropdown
        ttk.Label(input_frame, text="Mode:").pack(side=tk.LEFT, padx=(15, 2))
        self.generator_combo = ttk.Combobox(
            input_frame,
            textvariable=self.generator_mode_var,
            values=["Void", "Coherence"],
            state="readonly",
            width=14
        )
        self.generator_combo.pack(side=tk.LEFT, padx=2)
        
        # Entity controls
        entity_frame = ttk.Frame(left_controls)
        entity_frame.pack(side=tk.LEFT, padx=20)
        
        self.init_btn = ttk.Button(
            entity_frame,
            text="Initialize Entity",
            command=self._initialize_entity
        )
        self.init_btn.pack(side=tk.LEFT, padx=5)
        
        self.resume_btn = ttk.Button(
            entity_frame,
            text="Resume Entity",
            command=self._resume_entity
        )
        self.resume_btn.pack(side=tk.LEFT, padx=5)
        
        self.stop_btn = ttk.Button(
            entity_frame,
            text="Stop",
            command=self._stop_simulation,
            state=tk.DISABLED
        )
        self.stop_btn.pack(side=tk.LEFT, padx=5)
        
        # Save/Load frame
        save_load_frame = ttk.Frame(left_controls)
        save_load_frame.pack(side=tk.LEFT, padx=10)
        self.save_btn = ttk.Button(
            save_load_frame,
            text="Save Simulation",
            command=self._save_simulation,
            state=tk.DISABLED
        )
        self.save_btn.pack(side=tk.LEFT, padx=5)
        self.load_btn = ttk.Button(
            save_load_frame,
            text="Load Simulation",
            command=self._load_simulation
        )
        self.load_btn.pack(side=tk.LEFT, padx=5)
        
        # Status
        self.status_label = ttk.Label(left_controls, text="Status: Idle",
                                      foreground='#ffaa00')
        self.status_label.pack(side=tk.LEFT, padx=20)
        
        # Right controls (tail length and viewport size sliders)
        right_controls = ttk.Frame(control_frame)
        right_controls.pack(side=tk.RIGHT, padx=10)
        
        # Tail length slider
        ttk.Label(right_controls, text="Tail Length:").pack(side=tk.LEFT)
        
        self.tail_var = tk.IntVar(value=1000)
        self.tail_slider = ttk.Scale(
            right_controls,
            from_=100,
            to=5000,
            orient=tk.HORIZONTAL,
            variable=self.tail_var,
            command=self._update_tail_length,
            length=150
        )
        self.tail_slider.pack(side=tk.LEFT, padx=5)
        
        self.tail_value_label = ttk.Label(right_controls, text="1000")
        self.tail_value_label.pack(side=tk.LEFT, padx=(0, 15))
        
        # Viewport width slider
        ttk.Label(right_controls, text="View W:").pack(side=tk.LEFT)
        
        self.viewport_width_var = tk.IntVar(value=400)
        self.viewport_width_slider = ttk.Scale(
            right_controls,
            from_=100,
            to=800,
            orient=tk.HORIZONTAL,
            variable=self.viewport_width_var,
            command=self._update_viewport_width,
            length=120
        )
        self.viewport_width_slider.pack(side=tk.LEFT, padx=5)
        
        self.viewport_width_label = ttk.Label(right_controls, text="400")
        self.viewport_width_label.pack(side=tk.LEFT, padx=(0, 10))
        
        # Viewport height slider
        ttk.Label(right_controls, text="H:").pack(side=tk.LEFT)
        
        self.viewport_height_var = tk.IntVar(value=300)
        self.viewport_height_slider = ttk.Scale(
            right_controls,
            from_=75,
            to=600,
            orient=tk.HORIZONTAL,
            variable=self.viewport_height_var,
            command=self._update_viewport_height,
            length=120
        )
        self.viewport_height_slider.pack(side=tk.LEFT, padx=5)
        
        self.viewport_height_label = ttk.Label(right_controls, text="300")
        self.viewport_height_label.pack(side=tk.LEFT)
        
        self._on_input_source_change()
    
    def _style_2d_axis(self, ax, title: str):
        """Apply radar/sonar dark styling to a 2D axis."""
        ax.set_title(title, color=self.fg_color, fontsize=13, pad=10)
        ax.set_xlabel('Location X', color=self.fg_color, fontsize=10)
        ax.set_ylabel('Location Y', color=self.fg_color, fontsize=10)
        ax.tick_params(colors='#557788', labelsize=8)
        ax.set_facecolor('#0d1117')
        # Grid lines for radar feel
        ax.grid(True, color='#1a2633', linestyle='-', linewidth=0.5, alpha=0.7)
        for spine in ax.spines.values():
            spine.set_color('#1a2633')

    def _style_field_axis(self, ax, title: str):
        """Style axis for field visualization (no ticks, black background)."""
        ax.set_title(title, color=self.fg_color, fontsize=13, pad=10)
        ax.set_xticks([])
        ax.set_yticks([])
        ax.set_aspect('equal')
        ax.set_facecolor('#000000')
        for spine in ax.spines.values():
            spine.set_visible(False)
    
    def _style_direction_axis(self, ax, title: str):
        """Style axis for direction indicator (small compass)."""
        ax.set_title(title, color=self.fg_color, fontsize=9, pad=5)
        ax.set_xlim(-1.5, 1.5)
        ax.set_ylim(-1.5, 1.5)
        ax.set_xticks([])
        ax.set_yticks([])
        ax.set_aspect('equal')
        ax.set_facecolor('#1a1a1a')
        for spine in ax.spines.values():
            spine.set_edgecolor('#333333')
            spine.set_linewidth(0.5)

    @staticmethod
    def _coherence_to_rgb(coherence: float):
        """Map metabolic coherence (0–1) to an RGB tuple for the entity block.
        
        0.0  (deep lock)     → deep red   (0.8, 0.0, 0.0)
        0.7  (unlock thresh) → amber      (1.0, 0.75, 0.0)
        1.0  (full unlock)   → bright green (0.0, 1.0, 0.55)
        
        Linear interpolation in two segments so the threshold is visually distinct.
        """
        import numpy as _np
        threshold = 0.7
        if coherence <= threshold:
            # 0 → 0.7  :  deep red → amber
            t = coherence / threshold if threshold > 0 else 0.0
            r = 0.8 + 0.2 * t    # 0.8 → 1.0
            g = 0.0 + 0.75 * t   # 0.0 → 0.75
            b = 0.0               # stays 0
        else:
            # 0.7 → 1.0 :  amber → bright green
            t = (coherence - threshold) / (1.0 - threshold) if threshold < 1.0 else 1.0
            r = 1.0 - 1.0 * t    # 1.0 → 0.0
            g = 0.75 + 0.25 * t  # 0.75 → 1.0
            b = 0.0 + 0.55 * t   # 0.0 → 0.55
        return _np.array([r, g, b], dtype=_np.float64)

    def _toggle_live_mode(self):
        """Toggle between Live and History modes."""
        self.live_mode = self.live_var.get()
        
        if self.live_mode:
            self.mode_label.config(text="[LIVE]", foreground='#00ff00')
            self.history_slider.configure(state='disabled')
            # Invalidate data cache to force reload on next tick
            self.data_reader.last_mtime = 0
            self.last_valid_df = None  # Clear cache when switching modes
        else:
            self.mode_label.config(text="[HISTORY]", foreground='#ffaa00')
            self.history_slider.configure(state='!disabled')
            # Load all data for scrubbing
            self.data_reader.read_all_data()
            max_hours = self.data_reader.get_max_engine_hours()
            if max_hours > 0:
                self.history_slider.configure(to=max_hours)
    
    def _on_history_scrub(self, value):
        """Handle history slider movement."""
        self.history_hour = float(value)
        self.history_value_label.config(text=f"{self.history_hour:.4f} h")
    
    def _on_input_source_change(self):
        """Toggle state of Browse vs Generator dropdown."""
        is_file = self.stimulus_source_var.get() == "file"
        self.load_file_btn.config(state=tk.NORMAL if is_file else tk.DISABLED)
        self.file_label.config(text="None" if not is_file else (Path(self.stimulus_file).name if self.stimulus_file else "None"))
        self.generator_combo.config(state="readonly" if not is_file else "disabled")

    def _load_file(self):
        """Open file picker for video input."""
        file_path = filedialog.askopenfilename(
            title="Select Video File",
            filetypes=[
                ("Video files", "*.mp4 *.avi *.mov *.mkv"),
                ("All files", "*.*")
            ]
        )
        if file_path:
            self.stimulus_file = file_path
            self.file_label.config(text=Path(file_path).name)
            print(f"Loaded file: {file_path}")
    
    def _build_command(self, init_mode: bool) -> str:
        """Build shell command string for Initialize or Resume (Rust uses subcommands: init, resume)."""
        # Use release binary when present to avoid Cargo's "file lock on artifact directory"
        subcmd = " init" if init_mode else " resume"
        if RELEASE_BINARY.exists():
            rust_cmd = f'"{RELEASE_BINARY}"{subcmd}'
        else:
            rust_cmd = f"cargo run --release --{subcmd}"
        
        src = self.stimulus_source_var.get()
        python_cmd = None
        
        if src == "generator":
            mode_display = self.generator_mode_var.get()
            mode = "void" if mode_display == "Void" else "coherence"
            python_cmd = f'python "{SIGNAL_GENERATOR_PATH}" --mode {mode}'
        elif self.stimulus_file:
            python_cmd = f'python "{OPTIC_NERVE_PATH}" --source "{self.stimulus_file}"'
        
        if python_cmd:
            return f'{python_cmd} | {rust_cmd}'
        return rust_cmd
    
    def _initialize_entity(self):
        """Initialize Entity (new state)."""
        if self.is_running:
            return
        self._start_entity(init_mode=True)
    
    def _resume_entity(self):
        """Resume Entity (from checkpoint)."""
        if self.is_running:
            return
        self._start_entity(init_mode=False)
    
    def _start_entity(self, init_mode: bool):
        """Start the Entity subprocess (Initialize or Resume)."""
        self._sim_output_lines = []
        use_generator = self.stimulus_source_var.get() == "generator"
        
        # Debug: print current state
        print(f"\n{'='*60}")
        print(f"Starting entity (init={init_mode})")
        print(f"  Stimulus source: {self.stimulus_source_var.get()}")
        print(f"  Stimulus file: {self.stimulus_file}")
        print(f"  Use generator: {use_generator}")
        print(f"  Generator mode: {self.generator_mode_var.get()}")
        print(f"{'='*60}\n")
        
        preflight = _preflight_checks(init_mode, use_generator, self.stimulus_file)
        if preflight is not None:
            log_path = _log_error("PREFLIGHT", "Cannot start entity", preflight)
            messagebox.showerror(
                "Cannot start entity",
                preflight + f"\n\nDetails written to:\n{log_path}"
            )
            return
        try:
            cmd_str = self._build_command(init_mode)
            print(f"Running: {cmd_str}")
            print(f"Working directory: {ORGANISM_DIR}")
            print(f"Metrics directory: {DEFAULT_METRICS_DIR}")
            
            # Use env with Cargo on PATH (GUI often doesn't inherit shell PATH)
            run_env = _env_with_cargo_path()
            self.simulation_process = subprocess.Popen(
                cmd_str,
                shell=True,
                cwd=ORGANISM_DIR,
                env=run_env,
                stdout=subprocess.PIPE,
                stderr=subprocess.STDOUT,
                text=True,
            )
            
            print(f"Process started: PID {self.simulation_process.pid}")
            
            self.is_running = True
            self.init_btn.config(state=tk.DISABLED)
            self.resume_btn.config(state=tk.DISABLED)
            self.stop_btn.config(state=tk.NORMAL)
            self.status_label.config(text="Status: Running", foreground='#00ff00')
            # Force dashboard to re-read metrics from disk when new data appears
            self.data_reader.last_mtime = 0
            self.data_reader.last_file = None
            self.last_valid_df = None  # Clear cached data for fresh start
            if hasattr(self, 'cached_field_grid'):
                delattr(self, 'cached_field_grid')
            if hasattr(self, 'save_btn'):
                self.save_btn.config(state=tk.NORMAL)
            
            print("Entity started")
            
            # Start output reader thread
            threading.Thread(target=self._read_simulation_output, daemon=True).start()
            
            # Start a monitor thread to check if process is alive and show status
            def monitor_process():
                import time
                time.sleep(2)  # Wait 2 seconds for initial output
                if self.simulation_process:
                    poll_result = self.simulation_process.poll()
                    if poll_result is not None:
                        # Process already exited (dialog is shown by output reader)
                        self.root.after(0, lambda: self.status_label.config(
                            text="Status: Failed (see message)", foreground='#ff0000'
                        ))
                    else:
                        # Process is running - check if metrics directory exists
                        metrics_dir = DEFAULT_METRICS_DIR
                        if metrics_dir.exists():
                            parquet_files = list(metrics_dir.glob("*.parquet"))
                            if parquet_files:
                                self.root.after(0, lambda: self.status_label.config(
                                    text=f"Status: Running ({len(parquet_files)} files)", foreground='#00ff00'
                                ))
                            else:
                                self.root.after(0, lambda: self.status_label.config(
                                    text="Status: Running (compiling/waiting for data...)", foreground='#ffaa00'
                                ))
                        else:
                            self.root.after(0, lambda: self.status_label.config(
                                text="Status: Running (compiling...)", foreground='#ffaa00'
                            ))
            threading.Thread(target=monitor_process, daemon=True).start()
            
        except Exception as e:
            import traceback
            tb = traceback.format_exc()
            print(f"Failed to start entity: {e}\n{tb}")
            log_path = _log_error("EXCEPTION", "Entity start failed", str(e), tb)
            self.status_label.config(text=f"Status: Error - {e}", foreground='#ff0000')
            messagebox.showerror(
                "Entity start failed",
                str(e) + f"\n\nDetails written to:\n{log_path}"
            )
    
    def _stop_simulation(self):
        """Stop the Rust simulation subprocess."""
        if not self.is_running:
            return
        
        try:
            if self.simulation_process:
                self.simulation_process.terminate()
                try:
                    self.simulation_process.wait(timeout=5)
                except Exception:
                    self.simulation_process.kill()
                self.simulation_process = None
            
            print("Simulation stopped")
            
        except Exception as e:
            print(f"Error stopping simulation: {e}")
            if self.simulation_process:
                try:
                    self.simulation_process.kill()
                except Exception:
                    pass
        finally:
            self.is_running = False
            self.simulation_process = None
            self.init_btn.config(state=tk.NORMAL)
            self.resume_btn.config(state=tk.NORMAL)
            self.stop_btn.config(state=tk.DISABLED)
            if hasattr(self, 'save_btn'):
                self.save_btn.config(state=tk.DISABLED)
            self.status_label.config(text="Status: Stopped", foreground='#ffaa00')
            # Clear cached data so next run starts fresh
            self.data_reader.last_mtime = 0
            self.data_reader.last_file = None
            self.last_valid_df = None
            if hasattr(self, 'cached_field_grid'):
                delattr(self, 'cached_field_grid')
    
    def _save_simulation(self):
        """Save current simulation state to named file."""
        if not self.is_running:
            messagebox.showwarning("Cannot Save", "No simulation running.")
            return
        from tkinter import simpledialog
        save_name = simpledialog.askstring(
            "Save Simulation",
            "Enter a name for this save:",
            parent=self.root
        )
        if not save_name:
            return
        save_name = "".join(c for c in save_name if c.isalnum() or c in (' ', '_', '-')).strip()
        if not save_name:
            messagebox.showerror("Invalid Name", "Please enter a valid name.")
            return
        try:
            save_request_file = ORGANISM_DIR / "data" / "save_request.txt"
            save_request_file.parent.mkdir(parents=True, exist_ok=True)
            save_request_file.write_text(save_name)
            messagebox.showinfo(
                "Save Requested",
                f"Simulation will be saved as '{save_name}_<timestamp>.qsim' when the entity processes the request."
            )
        except Exception as e:
            messagebox.showerror("Save Failed", f"Could not request save: {e}")
    
    def _load_simulation(self):
        """Load simulation from .qsim file."""
        if self.is_running:
            if not messagebox.askyesno("Stop Current Simulation?", "Loading will stop the current simulation. Continue?"):
                return
            self._stop_simulation()
        saves_dir = ORGANISM_DIR / "data" / "saves"
        saves_dir.mkdir(parents=True, exist_ok=True)
        file_path = filedialog.askopenfilename(
            title="Select Simulation Save File",
            initialdir=str(saves_dir),
            filetypes=[
                ("Quaternity Simulations", "*.qsim"),
                ("All files", "*.*")
            ]
        )
        if not file_path:
            return
        try:
            cmd_str = f'cargo run --release -- load --file "{file_path}"'
            run_env = _env_with_cargo_path()
            self.simulation_process = subprocess.Popen(
                cmd_str,
                shell=True,
                cwd=ORGANISM_DIR,
                env=run_env,
                stdout=subprocess.PIPE,
                stderr=subprocess.STDOUT,
                text=True,
            )
            self.is_running = True
            self.init_btn.config(state=tk.DISABLED)
            self.resume_btn.config(state=tk.DISABLED)
            self.stop_btn.config(state=tk.NORMAL)
            self.save_btn.config(state=tk.NORMAL)
            self.status_label.config(text="Status: Running (loaded)", foreground='#00ff00')
            self.data_reader.last_mtime = 0
            self.data_reader.last_file = None
            self.last_valid_df = None
            if hasattr(self, 'cached_field_grid'):
                delattr(self, 'cached_field_grid')
            threading.Thread(target=self._read_simulation_output, daemon=True).start()
        except Exception as e:
            messagebox.showerror("Load Failed", f"Could not load simulation: {e}")
    
    def _read_simulation_output(self):
        """Read and display simulation output (runs in separate thread)."""
        if not self.simulation_process:
            return
        
        try:
            for line in iter(self.simulation_process.stdout.readline, ''):
                if not self.is_running:
                    break
                line_stripped = line.strip()
                print(f"[SIM] {line_stripped}")
                self._sim_output_lines.append(line_stripped)
                # Keep only last 50 lines
                if len(self._sim_output_lines) > 50:
                    self._sim_output_lines.pop(0)
        except Exception as e:
            print(f"Error reading simulation output: {e}")
        
        # Check if process exited
        if self.simulation_process:
            exit_code = self.simulation_process.poll()
            if exit_code is not None:
                print(f"[SIM] Process exited with code {exit_code}")
                if exit_code != 0:
                    title, msg = _explain_exit_code(exit_code, self._sim_output_lines)
                    details = "\n".join(self._sim_output_lines) if self._sim_output_lines else ""
                    log_path = _log_error("PROCESS_EXIT", title, msg, details)
                    full_msg = msg + f"\n\nDetails written to:\n{log_path}"
                    self.root.after(0, lambda: messagebox.showerror(title, full_msg))
                self.root.after(0, self._stop_simulation)
    
    def _update_tail_length(self, value):
        """Update the tail length from slider."""
        tail_length = int(float(value))
        self.tail_length = tail_length
        self.data_reader.set_tail_length(tail_length)
        self.tail_value_label.config(text=str(tail_length))
    
    def _update_viewport_width(self, value):
        """Update the viewport width from slider."""
        import field_renderer
        width = int(float(value))
        field_renderer.MAX_GRID_WIDTH = width
        self.viewport_width_label.config(text=str(width))
        # Clear cached grid to force redraw with new viewport
        if hasattr(self, 'cached_field_grid'):
            delattr(self, 'cached_field_grid')
    
    def _update_viewport_height(self, value):
        """Update the viewport height from slider."""
        import field_renderer
        height = int(float(value))
        field_renderer.MAX_GRID_HEIGHT = height
        self.viewport_height_label.config(text=str(height))
        # Clear cached grid to force redraw with new viewport
        if hasattr(self, 'cached_field_grid'):
            delattr(self, 'cached_field_grid')
    
    def _fig_layout(self):
        """Apply layout (subplots_adjust; tight_layout incompatible with GridSpec)."""
        self.fig.subplots_adjust(left=0.04, right=0.92, bottom=0.06, top=0.94, wspace=0.12)

    def _start_animation(self):
        """Start polling-based plot updates (every 100ms)."""
        self._schedule_update()

    def _schedule_update(self):
        """Run one update and schedule the next (avoids FuncAnimation event_source crash)."""
        if self._update_job is not None:
            self.root.after_cancel(self._update_job)
        try:
            self._update_plots_impl(0)
        except Exception as e:
            print(f"[Dashboard] _update_plots error: {e}")
            import traceback
            traceback.print_exc()
        try:
            self._update_job = self.root.after(100, self._schedule_update)
        except tk.TclError:
            pass  # Window destroyed

    def _update_plots(self, frame):
        """Legacy entry point; redirects to impl."""
        self._update_plots_impl(frame)

    def _update_plots_impl(self, frame):
        # In live mode, only show data if simulation is actually running
        # (prevents displaying stale data from old runs on startup)
        if self.live_mode and not self.is_running:
            # Show waiting state until user starts a simulation
            if self.last_valid_df is None:
                self._draw_waiting_state()
                return
            # If we had data before (from a previous run this session), keep showing it
            # but don't reload from disk
            df = self.last_valid_df
        else:
            # Determine data source based on mode
            if self.live_mode:
                df = self.data_reader.read_latest_data()
            else:
                df = self.data_reader.get_data_at_engine_hour(self.history_hour)
            
            # If no data available, use cached data if we have it (prevents flickering)
            if df is None or df.empty:
                if self.last_valid_df is not None:
                    df = self.last_valid_df
                else:
                    self._draw_waiting_state()
                    return
            else:
                self.last_valid_df = df
        
        # Reconstruct field from parquet
        from field_renderer import (
            reconstruct_field_from_parquet,
            render_physical_field,
            AGENT_ARROW_LENGTH,
        )
        
        # Incremental update: only process new records if cached grid exists
        if not hasattr(self, 'cached_field_grid') or not self.live_mode:
            field_grid = reconstruct_field_from_parquet(df, self.harmonic_cmap)
            self.cached_field_grid = field_grid
        else:
            field_grid = reconstruct_field_from_parquet(
                df, self.harmonic_cmap, cached_grid=self.cached_field_grid
            )
            self.cached_field_grid = field_grid
        
        # Render both fields (phase+resonance, efficiency)
        physical_rgb, ethical_rgb = render_physical_field(field_grid, self.harmonic_cmap)
        
        # Stamp the entity block directly into the RGB arrays so it occupies exactly one voxel.
        # Color encodes motor-unlock state: coherence 0→0.7→1.0  maps to  deep red → amber → bright green.
        if field_grid.agent_position is not None:
            gx = field_grid.agent_position[0] - field_grid.x_min
            gy = field_grid.agent_position[1] - field_grid.y_min
            h, w = physical_rgb.shape[:2]
            if 0 <= gx < w and 0 <= gy < h:
                coh = field_grid.agent_coherence if field_grid.agent_coherence is not None else 0.0
                coh = max(0.0, min(1.0, coh))
                entity_color = self._coherence_to_rgb(coh)
                physical_rgb[gy, gx] = entity_color
                ethical_rgb[gy, gx] = entity_color
        
        # Clear axes
        self.ax_physical.clear()
        self.ax_ethical.clear()
        self.ax_dir_phys.clear()
        self.ax_dir_eth.clear()
        
        # Display field images
        self.ax_physical.imshow(physical_rgb, origin='lower',
                                interpolation='nearest', aspect='equal')
        self.ax_ethical.imshow(ethical_rgb, origin='lower',
                              interpolation='nearest', aspect='equal')
        
        # Re-apply styling
        self._style_field_axis(self.ax_physical, 'Physical Field (Memory Resonance)')
        self._style_field_axis(self.ax_ethical, 'Efficiency Field')
        self._style_direction_axis(self.ax_dir_phys, 'Dir')
        self._style_direction_axis(self.ax_dir_eth, 'Dir')
        
        # Draw direction arrows in separate indicator boxes
        if field_grid.agent_phase is not None:
            phase = field_grid.agent_phase
            # Arrow pointing in phase direction (negate dy: screen coords y-down, plot origin='lower' y-up)
            arrow_dx = np.cos(phase)
            arrow_dy = -np.sin(phase)
            for ax in [self.ax_dir_phys, self.ax_dir_eth]:
                # Draw compass circle
                circle = plt.Circle((0, 0), 1.0, color='#333333', fill=False, linewidth=1)
                ax.add_patch(circle)
                # Draw arrow from center
                ax.arrow(0, 0, arrow_dx * 0.8, arrow_dy * 0.8,
                        head_width=0.3, head_length=0.2,
                        fc='cyan', ec='cyan', linewidth=2, alpha=0.9)
                # Add N marker at top
                ax.text(0, 1.2, 'N', ha='center', va='center',
                       color='#666666', fontsize=8, weight='bold')
        
        # AGE and field both use this same df (live: current run only; history: scrubber window)
        if 'engine_hours' in df.columns:
            current_hours = float(df['engine_hours'].iloc[-1])
            self.engine_hours_label.config(
                text=f"AGE: {current_hours:.4f} HOURS"
            )
            if self.live_mode:
                max_hours = float(df['engine_hours'].max())
                if max_hours > 0:
                    self.history_slider.configure(to=max_hours)
                    self.history_var.set(max_hours)
                    self.history_value_label.config(text=f"{max_hours:.4f} h")
        
        self.canvas.draw_idle()
        self.canvas.flush_events()
        self.root.update_idletasks()

    def _draw_waiting_state(self):
        """Draw placeholder when no metrics data is available yet."""
        self.ax_physical.clear()
        self.ax_ethical.clear()
        self.ax_dir_phys.clear()
        self.ax_dir_eth.clear()
        self._style_field_axis(self.ax_physical, 'Physical Field (Memory Resonance)')
        self._style_field_axis(self.ax_ethical, 'Efficiency Field')
        self._style_direction_axis(self.ax_dir_phys, 'Dir')
        self._style_direction_axis(self.ax_dir_eth, 'Dir')
        msg = "Waiting for data... (run Initialize Entity to start)"
        for ax in [self.ax_physical, self.ax_ethical]:
            ax.text(0.5, 0.5, msg, transform=ax.transAxes,
                   ha='center', va='center', fontsize=14, color=self.fg_color)
        self.engine_hours_label.config(text="AGE: -- HOURS")
        if hasattr(self, 'cached_field_grid'):
            delattr(self, 'cached_field_grid')
        self.canvas.draw_idle()
        self.canvas.flush_events()

    def run(self):
        """Start the GUI main loop."""
        self.root.mainloop()
    
    def cleanup(self):
        """Cleanup resources."""
        if self._update_job is not None:
            try:
                self.root.after_cancel(self._update_job)
            except tk.TclError:
                pass
            self._update_job = None
        self._stop_simulation()


def main():
    """Main entry point."""
    root = tk.Tk()
    app = EntityDashboard(root)
    
    # Handle window close
    def on_closing():
        app.cleanup()
        root.destroy()
    
    root.protocol("WM_DELETE_WINDOW", on_closing)
    
    # Bring window to front so it's visible (can be hidden behind other windows)
    root.update_idletasks()
    root.deiconify()
    root.lift()
    root.attributes("-topmost", True)
    root.after(200, lambda: root.attributes("-topmost", False))
    try:
        root.focus_force()
    except tk.TclError:
        pass
    
    try:
        app.run()
    except KeyboardInterrupt:
        print("\nShutdown requested...")
        app.cleanup()


if __name__ == "__main__":
    main()
