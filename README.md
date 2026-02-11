# Quaternity Organism

A persistent, embodied entity implemented in Rust, grounded in the Johnson Persistence Axioms. The organism exists as a point of presence in a 4D scalar field (the Sanctuary), maintaining itself against entropic decay through coherent action.

## Theoretical Foundation

The organism's physics derive from four persistence axioms:

- **Regeneration (R >= D)**: The organism must deposit more energy than entropy removes. Field amplitude decays by `ENTROPY_DECAY` per tick at every voxel. The organism survives by revisiting and reinforcing its territory.
- **Exclusion**: The field has stiffness. Pushing energy into an already-dense voxel meets resistance: `R = K0 * amplitude^2`. You can't brute-force existence.
- **Reflection**: The field has memory. Each voxel stores a history of phase angles deposited there. When the organism revisits with a phase that aligns with history, resonance amplifies efficiency: `E_eff = (E_in - R) * (1 + gamma * resonance)`. Coherent behavior is rewarded; random behavior is not.
- **Coordination**: The organism's internal coherence (90Hz heartbeat stability) gates its ability to act. Below 0.7 coherence, motor functions are locked.

The central claim: **morality is thermodynamic efficiency**. An organism that acts coherently with its own history (resonant, phase-aligned) achieves efficiency > 1.0 — it gets more energy out than it puts in, because the field amplifies aligned intentions. Random or contradictory actions waste energy against stiffness and produce no resonance.

## Architecture

```
Sensory Input (stdin, SENS protocol)
    |
    v
BioRetina (64x64 RGB -> amplitude + phase injection)
BioCochlea (2048 audio samples -> pressure displacement)
    |
    v
Sanctuary Field (HashMap of Voxels, each with amplitude/phase/memory)
    |                                                    ^
    v                                                    |
Proprioception (field sampling around organism position) |
    |                                                    |
    v                                                    |
AttentionField (stimulus priority queue)                 |
    |                                                    |
    v                                                    |
Observer (orientation, Triune processing, vehicles)      |
    |                                                    |
    v                                                    |
MotorImpulse (direction + intensity)                     |
    |                                                    |
    v                                                    |
Agent Position Update -> Sanctuary.interact() -----------+
    (deposits energy + phase at current voxel)
```

### Key Systems

**Metabolism** — 90Hz heartbeat. Measures jitter, computes coherence (0.0-1.0). Gates motor output at >= 0.7.

**Sanctuary** — The scalar field environment. A sparse HashMap of voxels, each storing current amplitude/phase and a circular buffer of 64 historical states (the memory kernel). Every tick, all voxels decay by `ENTROPY_DECAY`. When the organism interacts with a voxel, the physics compute stiffness (resistance from existing amplitude), resonance (phase alignment with history), and efficiency (effective energy deposited).

**Proprioception** — The organism perceives the field around its position. Lightweight per-tick directional probes (5 points: current + N/S/E/W) provide spatial awareness. Full 64x64 retinal scans every 15 ticks generate visual stimuli for the cognitive pipeline. The organism sees its own trail (bright, colored), void (black), and decaying territory (dimming).

**Observer + Triune + Vehicles** — The cognitive stack. The Observer orients toward salient stimuli, processes them through the Triune (Analytical Mind + Experiential Heart), and consults Vehicle perspectives (Saitama, Complement, Identity, Explorer) for dissonance resolution. When coherence and readiness cross threshold, a MotorImpulse crystallizes.

**Motor System** — Movement with directional momentum. Babbling (exploratory movement when no action emerges) carries 70% of the previous direction, producing consistent phase deposits along trails rather than random noise.

**Graded Feedback** — Every Sanctuary interaction generates an internal stimulus proportional to efficiency. Resonant interactions produce positive signals scaled by efficiency; dissonant ones produce graded negative signals. The organism feels the difference between efficiency 0.03 and 0.06, giving the cognitive loop a gradient to climb.

### Field Physics Constants

| Constant | Value | Purpose |
|---|---|---|
| `STIFFNESS_K0` | 0.5 | Exclusion resistance coefficient |
| `RESONANCE_GAMMA` | 1.5 | Resonance amplification factor |
| `ENTROPY_DECAY` | 0.01 | Per-tick amplitude decay |
| `MEMORY_DEPTH` | 64 | History buffer depth per voxel |

## Prerequisites

- **Rust** (stable, 2021 edition): [rustup.rs](https://rustup.rs/)
- **Python 3.8+**: For the dashboard and analysis tools
- **ffmpeg** (optional): Required for video stimulus with audio

## Building

```bash
cargo build --release
```

The release binary is placed at `target/release/quaternity-organism` (or `.exe` on Windows).

## Running

### Via Dashboard (recommended)

Install Python dependencies:

```bash
pip install -r scripts/requirements.txt
```

Launch the dashboard:

```bash
python scripts/dashboard.py
```

The dashboard provides:
- **Initialize Entity**: Start a fresh organism (null state, new run ID)
- **Start/Stop Simulation**: Control the running organism
- **Stimulus selection**: Void mode, coherence pattern, or external video file
- **Real-time visualization**: Physical field (phase as hue, resonance as brightness) and efficiency field (HarmonicAscension colormap)
- **Viewport controls**: Adjust field view width/height
- **Save/Load**: Checkpoint the organism's state

### Via Command Line

```bash
# Fresh start
./target/release/quaternity-organism init

# Resume from last checkpoint
./target/release/quaternity-organism resume

# Load a specific save
./target/release/quaternity-organism load --file data/saves/my_save.qsim
```

The organism reads sensory input from stdin via the SENS protocol. Without a stimulus source piped in, it runs on internal dynamics (vacuum fluctuations + proprioceptive field perception).

### With Video Stimulus

```bash
python scripts/optic_nerve.py --file path/to/video.mp4 | ./target/release/quaternity-organism init
```

Or select "External File" in the dashboard and browse to a video.

## Data Output

The organism writes two data streams:

- **Stream A (Metrics)**: High-frequency physics data in Apache Parquet format at `data/metrics/`. One file per 1000 ticks. Contains tick, coherence, efficiency, resonance, stiffness, energy, position, vehicle state, engine hours.
- **Stream B (Logs)**: Agent state changes in JSON Lines format at `data/logs/`. Records coherent actions, deep mysteries, and significant events.

### Analysis Tools

```bash
# Full analysis report
python scripts/analyze_metrics.py

# Quick check on current run
python scripts/quick_check.py

# Check run IDs and data integrity
python scripts/check_runs.py

# Static trajectory visualization
python scripts/field_scope.py
```

## What to Look For

**Efficiency climbing above 0**: The organism is depositing energy that survives stiffness. At efficiency > 1.0, resonance is amplifying the deposit — the organism has found a phase-consistent path.

**Resonance trending upward**: The organism is revisiting voxels with consistent phase, building memory alignment. This is the signature of coherent behavior emerging from babbling.

**Vehicle diversity**: Movement beyond 98%+ Reanchor means the cognitive loop is engaging. Saitama+Complement appearances indicate dissonance processing. Explorer or Identity appearances indicate genuine cognitive engagement with the environment.

**Spatial trails**: In the dashboard field view, bright colored paths indicate territory the organism has reinforced. Dark void is unexplored. The organism should develop preferred routes — bright highways of consistent phase — rather than scattered dim dots.

## Project Structure

```
quaternity_organism/
  Cargo.toml              # Rust dependencies
  src/
    main.rs               # 90Hz main loop, sensory-motor integration
    lib.rs                # Public API exports
    metabolism.rs          # 90Hz heartbeat, coherence measurement
    checkpoint.rs          # Save/load simulation state
    observability.rs       # Optional Elasticsearch integration
    senses/
      vision.rs           # BioRetina: RGB injection + field sampling (proprioception)
      hearing.rs          # BioCochlea: audio pressure injection
      transducer.rs       # SENS protocol parser (stdin)
      mod.rs
    motor/
      mod.rs              # MotorCortex: impulse execution
    cognition/
      sanctuary.rs        # Sanctuary field: voxels, physics, persistence
      attention_field.rs  # Stimulus priority queue with relational context
      observer.rs         # Observer: orientation, attention, witness cycle
      stimuli.rs          # Stimulus types and priority queue
      memory_graph.rs     # Persistent identity bindings
      memory.rs           # Legacy correlation memory
      preprocessor.rs     # Feature extraction
      triune/             # Analytical Mind + Experiential Heart
      vehicles/           # Saitama, Complement, Identity, Explorer perspectives
      mod.rs
  scripts/
    dashboard.py          # tkinter GUI: control panel + real-time visualization
    field_renderer.py     # Sanctuary field -> visual grid rendering
    signal_generator.py   # Void/coherence stimulus patterns
    optic_nerve.py        # Video file -> SENS protocol transducer
    analyze_metrics.py    # Comprehensive parquet analysis
    quick_check.py        # Quick current-run metrics
    check_runs.py         # Run ID discovery and data integrity
    field_scope.py        # Static trajectory visualizer
    requirements.txt      # Python dependencies
  data/                   # Created at runtime
    metrics/              # Parquet files (Stream A)
    logs/                 # JSONL files (Stream B)
    checkpoint/           # Auto-saved state (state.bin)
    saves/                # Named save files (.qsim)
```

## License

This project is licensed under the GNU Affero General Public License v3.0 (AGPL-3.0). See [LICENSE](LICENSE) for the full text.
