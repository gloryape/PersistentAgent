# Sanctuary Dashboard - Quick Start Guide

## Prerequisites

- **Python 3.8+** installed and on your PATH.
  If `python --version` fails, install from [python.org](https://www.python.org/downloads/) and check **"Add Python to PATH"**.
- **Rust** (stable): Install from [rustup.rs](https://rustup.rs/)

## Installation

From the **quaternity_organism** directory:

```bash
cd quaternity_organism

# Install Python dependencies (dashboard + analysis)
pip install -r scripts/requirements.txt

# Build the Rust organism (release mode)
cargo build --release
```

## Launch the Dashboard

```bash
python scripts/dashboard.py
```

(Use `python3` instead of `python` on Linux/macOS if needed.)

You should see the Mission Control window with:
- Two side-by-side field visualizations
- Control panel at the bottom
- Dark cyberpunk theme

## Running the Organism (resume testing)

1. **Launch the dashboard**: `python scripts/dashboard.py`
2. **Click "Initialize Entity"** — creates a fresh run (new run UUID, null state).
3. **Click "Start Simulation"** — starts the organism; the Rust binary runs in the background and writes metrics to `data/metrics/`.
4. After a few seconds the field view updates; AGE and Tick advance. You can now resume testing (vehicle distribution, efficiency, multi-agent behavior).
5. **Stop Simulation** when done; state is auto-saved so you can **Resume** later from the same run.

Multi-agent runs show multiple colored markers in the field (one per agent). The metrics bar shows "Agents: N" when `agent_id` is present in the data.

### With Video Stimulus

1. Select **"External File"** as the stimulus source
2. Click **"Browse"** and select a video file (MP4, AVI, MOV)
3. Click **"Start Simulation"**
4. The video frames and audio are streamed to the organism as sensory input

**Note**: Video stimulus requires `ffmpeg` installed for audio extraction, and `opencv-python-headless` (`pip install opencv-python-headless`).

## What You'll See

### Left Plot: Physical Field
- Phase encoded as hue, resonance as brightness
- The white dot is the organism's current position
- Bright colored trails = reinforced territory
- Dark regions = void or decayed

### Right Plot: Efficiency Field
- Custom **HarmonicAscension** colormap
- Red = low efficiency (survival mode)
- Green = equilibrium
- Violet/White = resonance amplification (efficiency > 1.0)

## Controls

- **Tail Length Slider**: Adjust history depth (100-5000 data points)
- **View W / H Sliders**: Adjust field viewport size
- **Stop Simulation**: Gracefully stops the organism (state is auto-saved)
- **Save/Load**: Checkpoint organism state to `.qsim` files

## Troubleshooting

### "Plots are empty"
- Ensure the simulation is running (status shows "Running")
- Wait a few seconds for data to be generated
- Check that `data/metrics/` directory contains `.parquet` files

### "Can't start simulation"
- Build the release binary first: `cargo build --release`
- Check that `target/release/quaternity-organism` (or `.exe`) exists

### Performance issues
- Reduce the **Tail Length** slider
- Reduce **View W** and **H** sliders
