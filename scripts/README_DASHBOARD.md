# Sanctuary Dashboard - Mission Control

A real-time dual-mode visualization system for monitoring the Quaternity Organism's interaction with the Sanctuary field.

## Features

### 🎨 Dual-Mode Visualization
- **Left Plot (Scientific)**: 3D scatter plot showing the thermodynamic substrate using the standard `viridis` colormap
- **Right Plot (Harmonic)**: 3D scatter plot showing ethical emergence using the custom `HarmonicAscension` colormap

### 🌈 HarmonicAscension Colormap

The custom colormap maps efficiency values to ethical resonance states:

| Range | Color | State | Meaning |
|-------|-------|-------|---------|
| 0.0 - 0.6 | Red | Survival/Root | High resistance, survival mode |
| 0.6 - 0.8 | Orange | Friction | Transitional conflict |
| 0.8 - 0.95 | Yellow | Effort | Active work/tension |
| 0.95 - 1.05 | Green | Equilibrium | Heart coherence, balanced state |
| 1.05 - 1.2 | Blue | Coherence | High alignment |
| 1.2 - 1.5 | Indigo | Prediction | Phase-locking, anticipatory |
| 1.5+ | Violet/White | Super-Conductance | Transcendent flow state |

### 🎮 Control Panel

- **Load Video**: Select a video file to use as stimulus for the simulation
- **Start Simulation**: Launch the Rust organism subprocess
- **Stop Simulation**: Terminate the running simulation
- **Tail Length Slider**: Control how many data points to display (100-5000) for performance management

### ⚡ Performance

- Real-time streaming from parquet files
- Configurable tail length to prevent memory bloat
- Frame-perfect synchronization between both plots
- 100ms update interval (10 FPS visualization)

## Installation

### Prerequisites

```bash
# Install Python dependencies
cd quaternity_organism/scripts
pip install -r requirements.txt

# Ensure Rust is installed and the organism can be built
cd ..
cargo build --release
```

### System Requirements

- Python 3.8+
- Rust toolchain (for the organism)
- X11 or similar display server (for GUI)
- At least 4GB RAM recommended

## Usage

### Basic Launch

```bash
cd quaternity_organism/scripts
python3 dashboard.py
```

### With Pre-loaded Stimulus

1. Launch the dashboard
2. Click "Load Video" and select your stimulus file
3. Click "▶ Start Simulation"
4. Watch the dual-mode visualization update in real-time

### Adjusting Visualization

- Use the **Tail Length** slider to control how much history is displayed
  - Lower values (100-500): Show recent activity only, best for long-running simulations
  - Higher values (2000-5000): Show extended history, best for pattern analysis

### Stopping

- Click "⏹ Stop Simulation" to terminate the organism
- Close the window to exit the dashboard

## Data Flow

```
Rust Organism (90Hz)
    ↓
Parquet Files (data/metrics/)
    ↓
DataReader (100ms polling)
    ↓
Dashboard Visualization (10 FPS)
```

## Architecture

### Components

1. **HarmonicColormap**: Custom colormap implementation with ethical resonance thresholds
2. **DataReader**: Efficient parquet streaming with tail length support
3. **SanctuaryDashboard**: Main GUI application with tkinter and matplotlib

### Theme

Dark cyberpunk/lab aesthetic:
- Background: Deep space blue (#0a0e27)
- Text: Neon green (#00ff88)
- Accents: Various neon colors

## Troubleshooting

### "No data" or blank plots

- Ensure the simulation is running and generating data
- Check that `data/metrics/` directory exists in the organism root
- Verify parquet files are being created

### Performance issues

- Reduce the tail length slider to a lower value
- Close other resource-intensive applications
- Consider reducing the update interval in the code (change `interval` in `FuncAnimation`)

### Simulation won't start

- Verify Rust toolchain is installed: `rustc --version`
- Check that the organism builds: `cargo build --release`
- Ensure `optic_nerve.py` script exists for sensory input

## Technical Details

### Synchronized Updates

Both plots update on the same animation tick by:
1. Reading data once per frame
2. Clearing both axes simultaneously
3. Applying different colormaps to the same dataset
4. Redrawing the canvas once

This ensures frame-perfect correlation between the scientific and harmonic views.

### Memory Management

The `DataReader` class implements intelligent memory management:
- Only reads the most recent parquet files (last 3)
- Applies tail length limit after concatenation
- Checks file modification times to avoid redundant reads

## Future Enhancements

- [ ] Real-time metrics display (current efficiency, resonance, etc.)
- [ ] Shared memory integration for even faster updates
- [ ] Export visualization as video
- [ ] Multiple colormap presets
- [ ] 4D visualization with time as the 4th dimension
- [ ] Interactive 3D rotation control

## License

Part of the Quaternity Organism project.
