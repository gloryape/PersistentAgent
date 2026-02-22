//! Entity - Main Entry Point
//!
//! A persistent physical entity defined by Johnson Persistence Axioms:
//! - Amplitude (A > 0): Entity maintains energy against vacuum decay
//! - Frequency (f = 90Hz): Entity processes distinct time steps
//! - Reflection: Entity's structure (Sanctuary) is geometric deformation from past interactions
//! - Regeneration (R >= D): Entity serializes state to disk to prevent entropic data loss
//!
//! Architecture Flow:
//! 1. Senses gather data (Retina, Cochlea)
//! 2. PreProcessor extracts features into Stimuli
//! 3. AttentionField manages relational context
//! 4. Observer orients (scans, enriches with memory)
//! 5. Observer attends (Triune processing, vehicle consultation, threshold check)
//! 6. MotorImpulse emerges when threshold crossed
//! 7. MemoryGraph records meaningful events

use clap::{Parser, Subcommand};
use image::DynamicImage;
use quaternity_organism::{
    authorize_action, AudioAnalysis, Metabolism, MotorCortex,
    Observer, VisualSignature, AudioSignature,
    Stimulus, StimulusSource, Rect,
    // New architecture
    AttentionField, MemoryGraph, MotorImpulse,
    // Sanctuary (4D Scalar Field Environment)
    Sanctuary,
    // Bio-Mimetic Sensory System
    BioRetina, BioCochlea, Transducer, SensoryFrame,
};
use quaternity_organism::cognition::{
    TriuneProcessor, VehicleSystem, WitnessOutcome,
    sanctuary::{direction_to_phase, intensity_to_energy, position_to_voxel, DomainMaze, DomainWall, STIFFNESS_K0},
    stimuli::MetabolicState,
};
use serde_json;
use uuid::Uuid;
#[cfg(feature = "observability")]
use quaternity_organism::observability::ThoughtIndexer;
use std::collections::{HashSet, VecDeque};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::path::PathBuf;
use std::thread;

use quaternity_organism::checkpoint::{self, save_simulation, OrganismCheckpoint};
use quaternity_organism::EnvironmentContext;

#[derive(Parser)]
#[command(name = "quaternity-organism")]
#[command(about = "Persistent physical entity (90Hz processing loop)")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Initialize a null-state Sanctuary (new Run UUID)
    Init,
    /// Deserialize the last saved state from disk
    Resume,
    /// Save current state to named file (use during runtime via dashboard or signal)
    Save {
        /// Name for this save (timestamp auto-appended)
        name: String,
    },
    /// Load state from a .qsim file
    Load {
        /// Path to .qsim file (e.g. data/saves/name_timestamp.qsim)
        #[arg(short, long)]
        file: PathBuf,
    },
}

// ═══════════════════════════════════════════════════════════════════════════
// ENVIRONMENTAL BEACONS
// Autonomous light sources in the Sanctuary — voxels that pulse with energy
// independently of the organism. The beacon builds phase history in its voxel
// through periodic energy injection, creating a resonance-capable structure
// the organism can discover, approach, and interact with.
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
struct Beacon {
    coords: (i32, i32, i32),
    energy: f32,
    phase: f32,
    pulse_interval: u32,
    max_amplitude: f32,
}

fn create_beacons() -> Vec<Beacon> {
    vec![
        Beacon {
            coords: (150, 80, 0),
            energy: 0.2,
            phase: std::f32::consts::FRAC_PI_4,
            pulse_interval: 15,
            max_amplitude: 0.8,
        },
    ]
}

/// Tracks recent positions and per-quadrant efficiency for vehicle EnvironmentContext.
const TRAJECTORY_CAPACITY: usize = 50;
const QUADRANT_EFFICIENCY_SAMPLES: usize = 8;

struct TrajectoryTracker {
    positions: VecDeque<(f32, f32)>,
    quadrant_efficiency: [[f32; QUADRANT_EFFICIENCY_SAMPLES]; 4],
    quadrant_eff_count: [usize; 4],
    quadrant_eff_idx: [usize; 4],
    quadrant_visit_count: [u32; 4],
    visited_voxels: HashSet<(i32, i32)>,
}

impl TrajectoryTracker {
    fn new() -> Self {
        Self {
            positions: VecDeque::with_capacity(TRAJECTORY_CAPACITY),
            quadrant_efficiency: [[0.0; QUADRANT_EFFICIENCY_SAMPLES]; 4],
            quadrant_eff_count: [0; 4],
            quadrant_eff_idx: [0; 4],
            quadrant_visit_count: [0; 4],
            visited_voxels: HashSet::new(),
        }
    }

    fn push_position(&mut self, x: f32, y: f32) {
        if self.positions.len() >= TRAJECTORY_CAPACITY {
            self.positions.pop_front();
        }
        self.positions.push_back((x, y));
    }

    /// Which quadrant the organism is heading toward (0=NW, 1=NE, 2=SW, 3=SE)
    fn current_heading_quadrant(&self) -> u8 {
        if self.positions.len() < 2 {
            return 0;
        }
        let last = self.positions.back().copied().unwrap_or((0.0, 0.0));
        let prev = self.positions.get(self.positions.len().saturating_sub(6)).copied().unwrap_or(last);
        let dx = last.0 - prev.0;
        let dy = last.1 - prev.1;
        if dx >= 0.0 && dy < 0.0 {
            1 // NE
        } else if dx >= 0.0 && dy >= 0.0 {
            3 // SE
        } else if dx < 0.0 && dy >= 0.0 {
            2 // SW
        } else {
            0 // NW
        }
    }

    /// 0.0 = random, 1.0 = dead straight
    fn heading_consistency(&self) -> f32 {
        if self.positions.len() < 4 {
            return 0.0;
        }
        let n = self.positions.len();
        let mut dots = 0.0f32;
        let mut count = 0usize;
        for i in 1..(n - 1).min(10) {
            let a = (
                self.positions[n - 1 - i].0 - self.positions[n - 2 - i].0,
                self.positions[n - 1 - i].1 - self.positions[n - 2 - i].1,
            );
            let b = (
                self.positions[n - 1].0 - self.positions[n - 2].0,
                self.positions[n - 1].1 - self.positions[n - 2].1,
            );
            let na = (a.0 * a.0 + a.1 * a.1).sqrt().max(1e-6);
            let nb = (b.0 * b.0 + b.1 * b.1).sqrt().max(1e-6);
            let dot = (a.0 * b.0 + a.1 * b.1) / (na * nb);
            dots += dot;
            count += 1;
        }
        if count == 0 {
            return 0.0;
        }
        ((dots / count as f32) + 1.0) * 0.5
    }

    fn record_interaction(&mut self, quadrant: u8, efficiency: f32, voxel: (i32, i32)) {
        let q = (quadrant as usize).min(3);
        self.quadrant_visit_count[q] = self.quadrant_visit_count[q].saturating_add(1);
        let idx = self.quadrant_eff_idx[q] % QUADRANT_EFFICIENCY_SAMPLES;
        self.quadrant_efficiency[q][idx] = efficiency;
        self.quadrant_eff_idx[q] += 1;
        if self.quadrant_eff_count[q] < QUADRANT_EFFICIENCY_SAMPLES {
            self.quadrant_eff_count[q] += 1;
        }
        self.visited_voxels.insert(voxel);
    }

    fn has_visited_voxel(&self, vx: i32, vy: i32) -> bool {
        self.visited_voxels.contains(&(vx, vy))
    }

    fn quadrant_recent_efficiency(&self) -> [Option<f32>; 4] {
        let mut out = [None; 4];
        for q in 0..4 {
            if self.quadrant_eff_count[q] == 0 {
                continue;
            }
            let n = self.quadrant_eff_count[q];
            let sum: f32 = self.quadrant_efficiency[q].iter().take(n).sum();
            out[q] = Some(sum / n as f32);
        }
        out
    }

    fn quadrant_visits(&self) -> [u32; 4] {
        self.quadrant_visit_count
    }
}

const NUM_AGENTS: usize = 3;
const WORLD_SIZE_VOXELS: i32 = 128;
const WORLD_SIZE_PX: f32 = (WORLD_SIZE_VOXELS * 10) as f32; // 1280.0

fn wrap_position(pos: &mut (f32, f32)) {
    pos.0 = ((pos.0 % WORLD_SIZE_PX) + WORLD_SIZE_PX) % WORLD_SIZE_PX;
    pos.1 = ((pos.1 % WORLD_SIZE_PX) + WORLD_SIZE_PX) % WORLD_SIZE_PX;
}

fn wrap_voxel(vx: i32, vy: i32) -> (i32, i32) {
    (
        ((vx % WORLD_SIZE_VOXELS) + WORLD_SIZE_VOXELS) % WORLD_SIZE_VOXELS,
        ((vy % WORLD_SIZE_VOXELS) + WORLD_SIZE_VOXELS) % WORLD_SIZE_VOXELS,
    )
}

struct AgentState {
    _id: usize,
    position: (f32, f32),
    phase: f32,
    observer: Observer,
    triune: TriuneProcessor,
    vehicles: VehicleSystem,
    attention_field: AttentionField,
    memory_graph: MemoryGraph,
    bio_retina: BioRetina,
    metabolism: Metabolism,
    trajectory_tracker: TrajectoryTracker,
    last_env_context: Option<EnvironmentContext>,
    efficiency_momentum: f32,
    somatic_update_count: u64,
    orient_counter: u32,
}

impl AgentState {
    fn new(id: usize, start_position: (f32, f32)) -> Self {
        Self {
            _id: id,
            position: start_position,
            phase: 0.0,
            observer: Observer::new(),
            triune: TriuneProcessor::new(),
            vehicles: VehicleSystem::new(),
            attention_field: AttentionField::new(),
            memory_graph: MemoryGraph::new(),
            bio_retina: BioRetina::new(16, 16),
            metabolism: Metabolism::new(),
            trajectory_tracker: TrajectoryTracker::new(),
            last_env_context: None,
            efficiency_momentum: 0.5,
            somatic_update_count: 0,
            orient_counter: 0,
        }
    }
}

fn outcome_to_vehicle(outcome: Option<&WitnessOutcome>) -> String {
    match outcome {
        Some(WitnessOutcome::Coherent(_)) => "Coherent".to_string(),
        Some(WitnessOutcome::NeedsPerspective(v)) => {
            v.iter().map(|vt| format!("{:?}", vt)).collect::<Vec<_>>().join("+")
        }
        Some(WitnessOutcome::DeepMystery(_)) => "Mystery".to_string(),
        Some(WitnessOutcome::ReanchorPresence) => "Reanchor".to_string(),
        None => "None".to_string(),
    }
}

fn main() {
    // Initialize logger with timestamp formatting
    env_logger::Builder::from_default_env()
        .format_timestamp_secs()
        .init();

    // Version info (embedded at build time)
    const VERSION: &str = env!("CARGO_PKG_VERSION");
    const GIT_HASH: &str = env!("GIT_HASH");
    const BUILD_TIME: &str = env!("BUILD_TIME");
    log::info!("[VERSION] quaternity-organism v{} (commit: {}, built: {})", VERSION, GIT_HASH, BUILD_TIME);

    let cli = Cli::parse();

    // Save must be requested during runtime (dashboard writes save_request.txt or sends signal)
    if let Command::Save { .. } = cli.command {
        log::error!("Save command is for runtime only. Use dashboard 'Save Simulation' or signal.");
        std::process::exit(1);
    }

    // Initialize Sanctuary (and optional organism state when Loading)
    let (mut sanctuary, loaded_organism): (Sanctuary, Option<OrganismCheckpoint>) = match cli.command {
        Command::Init => {
            log::info!("Entity initialized (null state)");
            let s = match Sanctuary::with_persistence() {
                Ok(s) => {
                    log::info!("Sanctuary initialized with Dual-Stream persistence");
                    log::info!("   Run ID: {}", s.get_run_id());
                    log::info!("   Stream A: Parquet (data/metrics/)");
                    log::info!("   Stream B: JSONL (data/logs/)");
                    s
                }
                Err(e) => {
                    log::warn!("Failed to enable Sanctuary persistence: {}. Using memory-only mode.", e);
                    Sanctuary::new()
                }
            };
            (s, None)
        }
        Command::Resume => {
            let s = match Sanctuary::deserialize_state() {
                Ok(s) => {
                    log::info!("State restored. Run ID: {}", s.get_run_id());
                    log::info!("Sanctuary restored with Dual-Stream persistence");
                    s
                }
                Err(e) => {
                    log::error!("Regeneration failed: {}. Entropy has won.", e);
                    std::process::exit(1);
                }
            };
            (s, None)
        }
        Command::Load { file } => {
            let path = if file.is_absolute() { file } else { PathBuf::from("data").join("saves").join(&file) };
            match checkpoint::load_simulation(&path) {
                Ok((s, org)) => {
                    log::info!("Simulation loaded from: {}", path.display());
                    (s, Some(org))
                }
                Err(e) => {
                    log::error!("Load failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Command::Save { .. } => unreachable!(),
    };
    
    // Set vacuum geometry: domain maze with phase-winding walls
    use std::f32::consts::PI;
    sanctuary.set_terrain(Box::new(DomainMaze {
        rho_0: 0.1,
        theta_base: 0.0,
        alpha: 8.0,
        walls: vec![
            // === Central maze (around agent_0 spawn at voxel 96,54) ===
            // Horizontal wall south of center with gap at x~92-100
            DomainWall { x1: 70.0, y1: 58.0, x2: 92.0, y2: 58.0,
                         delta_theta: PI, width: 1.5 },
            DomainWall { x1: 100.0, y1: 58.0, x2: 125.0, y2: 58.0,
                         delta_theta: PI, width: 1.5 },
            // Vertical wall east of center with gap at y~52-60
            DomainWall { x1: 108.0, y1: 35.0, x2: 108.0, y2: 52.0,
                         delta_theta: PI, width: 1.5 },
            DomainWall { x1: 108.0, y1: 60.0, x2: 108.0, y2: 80.0,
                         delta_theta: PI, width: 1.5 },
            // Diagonal funnel northwest
            DomainWall { x1: 82.0, y1: 42.0, x2: 92.0, y2: 52.0,
                         delta_theta: PI * 0.7, width: 2.0 },

            // === Northwest quadrant walls (near agent_1 spawn at voxel 40,30) ===
            // Horizontal wall with gap at x~38-44 (centered on agent_1 spawn x=40)
            DomainWall { x1: 20.0, y1: 32.0, x2: 38.0, y2: 32.0,
                         delta_theta: PI, width: 1.5 },
            DomainWall { x1: 44.0, y1: 32.0, x2: 55.0, y2: 32.0,
                         delta_theta: PI, width: 1.5 },
            // Vertical wall with gap at y=24-30 (visible from agent_1 spawn y=30)
            DomainWall { x1: 48.0, y1: 15.0, x2: 48.0, y2: 24.0,
                         delta_theta: PI, width: 1.5 },

            // === Southwest quadrant walls (near agent_2 spawn at voxel 60,80) ===
            DomainWall { x1: 35.0, y1: 85.0, x2: 75.0, y2: 85.0,
                         delta_theta: PI, width: 1.5 },
            DomainWall { x1: 60.0, y1: 70.0, x2: 60.0, y2: 82.0,
                         delta_theta: PI, width: 1.5 },

            // === Cross-grid walls to create corridors ===
            // Vertical barrier mid-grid with gap at y~55-65
            DomainWall { x1: 64.0, y1: 10.0, x2: 64.0, y2: 50.0,
                         delta_theta: PI, width: 1.5 },
            DomainWall { x1: 64.0, y1: 65.0, x2: 64.0, y2: 120.0,
                         delta_theta: PI, width: 1.5 },
            // Horizontal barrier mid-grid with gap at x~60-72
            DomainWall { x1: 5.0, y1: 64.0, x2: 55.0, y2: 64.0,
                         delta_theta: PI, width: 1.5 },
            DomainWall { x1: 75.0, y1: 64.0, x2: 120.0, y2: 64.0,
                         delta_theta: PI, width: 1.5 },
        ],
    }));

    // Initialize transducer from stdin (SENS protocol)
    let mut transducer = Transducer::from_stdin();
    log::info!("Transducer initialized (reading from stdin)");
    
    log::info!("Retina initialized: 16x16 receptor grid (per agent)");
    
    // Initialize Cochlea (Resonant Membrane)
    let mut bio_cochlea = BioCochlea::default();
    log::info!("Cochlea initialized: membrane at {:?}", bio_cochlea.drum_location());

    // Initialize Motor Cortex (Virtual Hand)
    let mut motor = match MotorCortex::new() {
        Ok(m) => {
            log::info!("Motor Cortex initialized - Movement activated");
            m
        }
        Err(e) => {
            log::warn!("Failed to initialize motor cortex: {}. Continuing without movement.", e);
            MotorCortex::default()
        }
    };

    // Set up Ctrl+C handler for graceful shutdown (Regeneration: R >= D)
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        log::info!("Shutdown signal received. Serializing state...");
        r.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl+C handler");

    // Save request state (dashboard writes data/save_request.txt with the save name)
    static SAVE_REQUESTED: AtomicBool = AtomicBool::new(false);
    static SAVE_NAME: std::sync::OnceLock<Mutex<Option<String>>> = std::sync::OnceLock::new();
    SAVE_NAME.get_or_init(|| Mutex::new(None));

    // Beacon toggle (dashboard writes data/beacon_enabled.txt with "0" or "1")
    static BEACONS_ENABLED: AtomicBool = AtomicBool::new(false);

    let running_save = Arc::clone(&running);
    thread::spawn(move || {
        let request_path = PathBuf::from("data").join("save_request.txt");
        let beacon_path = PathBuf::from("data").join("beacon_enabled.txt");
        while running_save.load(Ordering::SeqCst) {
            thread::sleep(Duration::from_secs(1));
            if let Ok(contents) = std::fs::read_to_string(&request_path) {
                let name = contents.trim().to_string();
                if !name.is_empty() {
                    if let Some(mu) = SAVE_NAME.get() {
                        if let Ok(mut guard) = mu.lock() {
                            *guard = Some(name);
                            SAVE_REQUESTED.store(true, Ordering::SeqCst);
                        }
                    }
                    let _ = std::fs::remove_file(&request_path);
                }
            }
            if let Ok(contents) = std::fs::read_to_string(&beacon_path) {
                match contents.trim() {
                    "0" => BEACONS_ENABLED.store(false, Ordering::SeqCst),
                    "1" => BEACONS_ENABLED.store(true, Ordering::SeqCst),
                    _ => {}
                }
            }
        }
    });

    // Initialize observability (optional, via ELASTICSEARCH_URL env var)
    #[cfg(feature = "observability")]
    let (observability, rt) = {
        let es_url = std::env::var("ELASTICSEARCH_URL").ok();
        let rt = tokio::runtime::Runtime::new().unwrap();
        let indexer = rt.block_on(ThoughtIndexer::new(es_url))
            .unwrap_or_else(|e| {
                log::warn!("Failed to initialize observability: {}", e);
                rt.block_on(ThoughtIndexer::new(None)).unwrap()
            });
        (indexer, rt)
    };

    let target_interval = Duration::from_nanos(11_111_111); // 11.11ms for 90Hz

    // Initialize agents at different starting positions in the maze.
    // Voxel coords: agent 0 at (96,54), agent 1 at (80,45), agent 2 at (120,70).
    let start_positions: [(f32, f32); NUM_AGENTS] = [
        (960.0, 540.0),   // voxel (96, 54) — center of maze
        (400.0, 300.0),   // voxel (40, 30) — northwest quadrant
        (600.0, 800.0),   // voxel (60, 80) — southwest quadrant
    ];
    let mut agents: Vec<AgentState> = (0..NUM_AGENTS)
        .map(|i| AgentState::new(i, start_positions[i]))
        .collect();

    // Restore loaded organism state into agent 0 if applicable
    let mut tick_count = if let Some(ref org) = loaded_organism {
        agents[0].metabolism.set_coherence(org.coherence);
        agents[0].position = org.agent_position_pixels;
        agents[0].phase = org.agent_phase;
        org.tick_count
    } else {
        0u32
    };

    const SOMATIC_DECAY: f32 = 0.85;

    // Log version to Stream B for quick_check diagnostics
    sanctuary.log_event(
        "version",
        &format!("Build: v{} (commit {}, built {})", VERSION, GIT_HASH, BUILD_TIME),
        serde_json::json!({
            "version": VERSION,
            "git_hash": GIT_HASH,
            "build_time": BUILD_TIME,
        }),
    );

    log::info!("Starting 90Hz processing loop with {} agents...", NUM_AGENTS);
    log::info!("Target interval: {:.2}ms (90Hz)", target_interval.as_secs_f64() * 1000.0);
    log::info!("Observer Architecture activated");
    for (i, agent) in agents.iter().enumerate() {
        log::info!("   Agent {}: start=({:.0}, {:.0})", i, agent.position.0, agent.position.1);
    }
    log::info!("---");

    let orient_interval = 5u32;
    let mut last_frame: Option<SensoryFrame> = None;

    let beacons = create_beacons();

    // Main 90Hz loop
    loop {
        if !running.load(Ordering::SeqCst) {
            break;
        }

        let loop_start = Instant::now();
        tick_count += 1;

        // Handle save request (uses agent 0 state)
        if SAVE_REQUESTED.swap(false, Ordering::SeqCst) {
            let name = SAVE_NAME.get().and_then(|m| m.lock().ok().and_then(|mut g| g.take()))
                .unwrap_or_else(|| format!("autosave_{}", tick_count));
            if let Err(e) = save_simulation(
                &sanctuary,
                agents[0].position,
                agents[0].phase,
                agents[0].metabolism.get_coherence(),
                tick_count,
                &name,
            ) {
                log::error!("Save failed: {}", e);
            }
        }

        // ═══════════════════════════════════════════════════════════════════
        // SHARED: SENSORY GATHERING (once per tick)
        // ═══════════════════════════════════════════════════════════════════
        let frame = match transducer.read_frame() {
            Ok(f) => {
                last_frame = Some(f.clone());
                Some(f)
            }
            Err(_) => last_frame.clone(),
        };

        let mut audio_volume = 0.0;
        let mut audio_entropy = 0.5;

        if let Some(ref sensory_frame) = frame {
            agents[0].bio_retina.inject_into_sanctuary(&sensory_frame.rgb_grid, &mut sanctuary);
            bio_cochlea.inject_into_sanctuary(&sensory_frame.audio_samples, &mut sanctuary);

            audio_volume = sensory_frame.audio_samples.iter()
                .map(|&s| s.abs())
                .sum::<f32>() / sensory_frame.audio_samples.len() as f32;

            audio_entropy = if audio_volume > 0.1 {
                0.3 + (audio_volume * 0.7) as f64
            } else {
                0.0
            };
        }

        // ═══════════════════════════════════════════════════════════════════
        // SHARED: ENVIRONMENTAL BEACONS (once per tick)
        // ═══════════════════════════════════════════════════════════════════
        if BEACONS_ENABLED.load(Ordering::SeqCst) {
            for beacon in &beacons {
                if tick_count as u64 % beacon.pulse_interval as u64 == 0 {
                    if sanctuary.density_at(beacon.coords) < beacon.max_amplitude {
                        sanctuary.inject_beacon(
                            beacon.coords.0, beacon.coords.1, beacon.coords.2,
                            beacon.energy, beacon.phase, tick_count as u64,
                        );
                    }
                }
            }
        }

        // ═══════════════════════════════════════════════════════════════════
        // PER-AGENT PROCESSING
        // Each agent perceives, thinks, and acts through the shared Sanctuary.
        // ═══════════════════════════════════════════════════════════════════
        for agent_idx in 0..agents.len() {
            let agent = &mut agents[agent_idx];

            sanctuary.set_agent_id(&format!("agent_{}", agent_idx));
            sanctuary.set_agent_position((agent.position.0 as i32, agent.position.1 as i32, 0));

            let _coherence_tick = agent.metabolism.tick();
            agent.orient_counter += 1;
            agent.trajectory_tracker.push_position(agent.position.0, agent.position.1);
            agent.last_env_context = None;

            if audio_volume > 0.5 {
                agent.metabolism.apply_audio_stress(audio_volume as f64, audio_entropy, 0.0);
            }

            // PROPRIOCEPTION: each agent perceives from its own position
            let agent_voxel = position_to_voxel(agent.position.0, agent.position.1);

            let probe_radius = 3i32;
            let here_amp = sanctuary.density_at((agent_voxel.0, agent_voxel.1, 0));
            let north_amp = sanctuary.density_at((agent_voxel.0, agent_voxel.1 - probe_radius, 0));
            let south_amp = sanctuary.density_at((agent_voxel.0, agent_voxel.1 + probe_radius, 0));
            let east_amp  = sanctuary.density_at((agent_voxel.0 + probe_radius, agent_voxel.1, 0));
            let west_amp  = sanctuary.density_at((agent_voxel.0 - probe_radius, agent_voxel.1, 0));

            let max_dir_amp = north_amp.max(south_amp).max(east_amp).max(west_amp);
            let min_dir_amp = north_amp.min(south_amp).min(east_amp).min(west_amp);
            let directional_contrast = if max_dir_amp > 0.001 {
                (max_dir_amp - min_dir_amp) / max_dir_amp
            } else {
                0.0
            };

            let local_amplitudes = [here_amp, north_amp, south_amp, east_amp, west_amp];
            let mean_amp = local_amplitudes.iter().sum::<f32>() / 5.0;
            let variance = local_amplitudes.iter()
                .map(|a| (a - mean_amp).powi(2))
                .sum::<f32>() / 5.0;
            let visual_entropy = (variance.sqrt() / 2.0).min(1.0) as f64;

            if visual_entropy > 0.8 {
                agent.metabolism.apply_visual_stress(visual_entropy);
            }

            // Full proprioceptive scan every 15 ticks
            if tick_count % 15 == 0 {
                let proprio_pixels = agent.bio_retina.sample_from_sanctuary_wrapped(
                    &sanctuary,
                    (agent_voxel.0, agent_voxel.1),
                    Some(WORLD_SIZE_VOXELS),
                );

                let mut total_brightness = 0.0f32;
                let mut bright_pixels = 0u32;
                let mut total_pixels = 0u32;
                let mut quadrant_brightness = [0.0f32; 4];
                let mut quadrant_counts = [0u32; 4];
                let mut quadrant_stiffness_sum = [0.0f32; 4];
                let mut quadrant_stiffness_count = [0u32; 4];
                let mut quadrant_amplitude_sum = [0.0f32; 4];
                let mut quadrant_novelty_sum = [0.0f32; 4];
                let mut quadrant_novelty_count = [0u32; 4];
                let mut has_novel_structure = [false; 4];
                let mut has_local_phase_history = [false; 4];
                let (pw, ph) = agent.bio_retina.resolution();
                let half_w = pw / 2;
                let half_h = ph / 2;
                let half_wi = half_w as i32;
                let half_hi = half_h as i32;

                for (y, row) in proprio_pixels.iter().enumerate() {
                    for (x, &[r, g, b]) in row.iter().enumerate() {
                        let lum = 0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32;
                        total_brightness += lum;
                        total_pixels += 1;
                        if lum > 10.0 { bright_pixels += 1; }

                        let qi = if (y as u32) < half_h {
                            if (x as u32) < half_w { 0 } else { 1 }
                        } else {
                            if (x as u32) < half_w { 2 } else { 3 }
                        };
                        quadrant_brightness[qi] += lum;
                        quadrant_counts[qi] += 1;

                        let (vx, vy) = wrap_voxel(
                            agent_voxel.0 + (x as i32 - half_wi),
                            agent_voxel.1 + (y as i32 - half_hi),
                        );
                        let total_amp = sanctuary.density_at((vx, vy, 0));
                        let stiff = STIFFNESS_K0 * total_amp * total_amp;
                        quadrant_stiffness_sum[qi] += stiff;
                        quadrant_stiffness_count[qi] += 1;
                        quadrant_amplitude_sum[qi] += total_amp;

                        let (terrain_amp, _) = sanctuary.terrain_at(vx, vy);
                        let excess_amp = total_amp - terrain_amp;
                        if excess_amp > 0.1 && !agent.trajectory_tracker.has_visited_voxel(vx, vy) {
                            has_novel_structure[qi] = true;
                        }

                        // Complement trigger: local phase history / wakes (voxel has history + excitation)
                        if excess_amp > 0.05 {
                            if let Some(voxel) = sanctuary.get_voxel((vx, vy, 0)) {
                                if !voxel.history.is_empty() {
                                    has_local_phase_history[qi] = true;
                                }
                            }
                        }

                        // Subjective novelty modulated by luminance.
                        // Dark voxels (energy locked in phase winding) are not novel —
                        // there is nothing free to explore. Bright voxels (free amplitude)
                        // are potential. The organism sees luminance; novelty must match.
                        let terrain_luminance = sanctuary.luminance_at((vx, vy, 0));
                        let rho_0 = 0.1_f32; // must match DomainMaze.rho_0
                        let luminance_factor = (terrain_luminance / rho_0).clamp(0.0, 1.0);

                        let novelty = if let Some(voxel) = sanctuary.get_voxel((vx, vy, 0)) {
                            if voxel.history.is_empty() {
                                luminance_factor
                            } else {
                                let resonance = voxel.calculate_resonance(agent.phase);
                                let visit_novelty = 1.0 - (resonance + 1.0) * 0.5;
                                visit_novelty * luminance_factor
                            }
                        } else {
                            luminance_factor
                        };
                        quadrant_novelty_sum[qi] += novelty;
                        quadrant_novelty_count[qi] += 1;
                    }
                }

                let mean_brightness = if total_pixels > 0 { total_brightness / total_pixels as f32 } else { 0.0 };
                let coverage = if total_pixels > 0 { bright_pixels as f32 / total_pixels as f32 } else { 0.0 };

                for i in 0..4 {
                    if quadrant_counts[i] > 0 {
                        quadrant_brightness[i] /= quadrant_counts[i] as f32;
                    }
                }

                let mut quadrant_stiffness = [0.0f32; 4];
                let mut quadrant_amplitude = [0.0f32; 4];
                let mut quadrant_novelty = [0.5f32; 4];
                for i in 0..4 {
                    if quadrant_stiffness_count[i] > 0 {
                        quadrant_stiffness[i] = quadrant_stiffness_sum[i] / quadrant_stiffness_count[i] as f32;
                        quadrant_amplitude[i] = quadrant_amplitude_sum[i] / quadrant_stiffness_count[i] as f32;
                    }
                    if quadrant_novelty_count[i] > 0 {
                        quadrant_novelty[i] = quadrant_novelty_sum[i] / quadrant_novelty_count[i] as f32;
                    }
                }
                let visits = agent.trajectory_tracker.quadrant_visits();
                let brightest_quadrant_u8 = quadrant_brightness
                    .iter()
                    .enumerate()
                    .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
                    .map(|(i, _)| i as u8)
                    .unwrap_or(0);
                agent.last_env_context = Some(EnvironmentContext {
                    quadrant_stiffness,
                    quadrant_amplitude,
                    quadrant_visit_count: visits,
                    quadrant_recent_efficiency: agent.trajectory_tracker.quadrant_recent_efficiency(),
                    current_heading_quadrant: agent.trajectory_tracker.current_heading_quadrant(),
                    heading_consistency: agent.trajectory_tracker.heading_consistency(),
                    brightest_quadrant: brightest_quadrant_u8,
                    has_novel_structure,
                    quadrant_novelty,
                    agent_voxel: (agent_voxel.0, agent_voxel.1, 0),
                    has_local_phase_history,
                });

                let scan_contrast = {
                    let sum: f32 = quadrant_brightness.iter().sum();
                    let max_q = quadrant_brightness.iter().cloned().fold(0.0f32, f32::max);
                    if sum > 0.001 {
                        ((max_q / sum) - 0.25) / 0.75
                    } else {
                        0.0
                    }
                };

                if agent_idx == 0 {
                    log::info!(
                        "[PROPRIO A0] tick={} cov={:.4} contrast={:.4} dir={:.4} mean={:.1}",
                        tick_count, coverage, scan_contrast, directional_contrast, mean_brightness
                    );
                }

                if coverage > 0.0 || scan_contrast > 0.05 {
                    let prop_salience = if coverage > 0.0 {
                        let presence = 0.3_f32;
                        let gradient = scan_contrast.min(1.0) * 0.7;
                        (presence + gradient).min(1.0) as f64
                    } else if scan_contrast > 0.05 {
                        (scan_contrast * 0.5).min(1.0) as f64
                    } else {
                        0.0
                    };
                    let prop_novelty = if coverage > 0.0 {
                        (0.2 + scan_contrast).min(1.0) as f64
                    } else {
                        0.0
                    };

                    let brightest_quadrant = quadrant_brightness
                        .iter()
                        .enumerate()
                        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
                        .map(|(i, _)| i)
                        .unwrap_or(0);

                    let proprioceptive_stimulus = Stimulus {
                        id: Uuid::new_v4(),
                        source: StimulusSource::Proprioceptive {
                            quadrant_brightness,
                            coverage: coverage as f32,
                            directional_contrast: scan_contrast,
                            brightest_quadrant,
                            mean_brightness: mean_brightness as f32,
                        },
                        urgency: prop_salience.min(1.0),
                        novelty: prop_novelty.min(1.0),
                        salience: prop_salience.min(1.0),
                        timestamp: Instant::now(),
                        attention_count: 0,
                    };
                    agent.attention_field.add(proprioceptive_stimulus);
                    agent.observer.signal_new_input();

                    if agent_idx == 0 {
                        sanctuary.log_event(
                            "proprio",
                            "Proprioceptive stimulus: field has visible structure",
                            serde_json::json!({
                                "agent": agent_idx,
                                "coverage": coverage,
                                "scan_contrast": scan_contrast,
                                "salience": prop_salience,
                                "novelty": prop_novelty,
                                "mean_brightness": mean_brightness
                            }),
                        );
                    }
                }
            }

            // ORIENTATION
            let should_orient = agent.orient_counter >= orient_interval
                || agent.observer.state.is_stagnant()
                || agent.observer.state.current_focus.is_none();

            if should_orient && !agent.attention_field.is_empty() {
                agent.observer.orient(&mut agent.attention_field, &agent.memory_graph);
                agent.orient_counter = 0;

                if tick_count % 100 == 0 {
                    agent.memory_graph.decay_links();
                }
            }

            // ENGAGEMENT
            let coherence = agent.metabolism.get_coherence();
            let authorized = authorize_action(coherence);
            let mut emerged_impulse: Option<MotorImpulse> = None;

            if authorized && !agent.attention_field.is_empty() {
                if let Some(target) = agent.attention_field.peek_highest_priority().cloned() {
                    agent.attention_field.mark_attending(&target.raw.id);

                    if let Some(impulse) = agent.observer.attend(
                        &target, &mut agent.triune, &agent.vehicles,
                        &agent.memory_graph, agent.last_env_context.as_ref(),
                        agent.efficiency_momentum,
                    ) {
                        emerged_impulse = Some(impulse.clone());
                        agent.attention_field.unmark_attending(&target.raw.id);

                        if let Some(ref outcome) = agent.observer.last_outcome {
                            match outcome {
                                WitnessOutcome::Coherent(_) => sanctuary.set_vehicle("Coherent"),
                                WitnessOutcome::NeedsPerspective(v) => {
                                    let names: Vec<_> = v.iter().map(|v| format!("{:?}", v)).collect();
                                    sanctuary.set_vehicle(&names.join("+"));
                                }
                                WitnessOutcome::DeepMystery(_) => sanctuary.set_vehicle("Mystery"),
                                WitnessOutcome::ReanchorPresence => sanctuary.set_vehicle("Reanchor"),
                            }
                        }

                        if let Some(WitnessOutcome::Coherent(_)) = &agent.observer.last_outcome {
                            agent.memory_graph.record_coherent_event(
                                &target, agent.observer.state.coherence,
                                agent.observer.state.inhibition,
                                agent.triune.experiential.recent_trend().0,
                                Some(impulse.clone()),
                            );
                            if agent_idx == 0 {
                                sanctuary.log_event(
                                    "coherent_action",
                                    &format!("Agent {} action with coherence {:.2}", agent_idx, agent.observer.state.coherence),
                                    serde_json::json!({
                                        "agent": agent_idx,
                                        "readiness": agent.observer.state.readiness,
                                        "coherence": agent.observer.state.coherence,
                                        "modality": format!("{:?}", impulse.modality),
                                        "intensity": impulse.intensity,
                                    }),
                                );
                            }
                        }

                        if let Some(WitnessOutcome::DeepMystery(_)) = &agent.observer.last_outcome {
                            agent.memory_graph.record_deep_mystery(
                                &target, agent.observer.state.coherence,
                                agent.observer.state.inhibition,
                                agent.triune.experiential.recent_trend().0,
                            );
                        }

                        agent.attention_field.remove(&target.raw.id);
                    } else {
                        agent.attention_field.unmark_attending(&target.raw.id);
                    }
                }
            }

            // MOTOR EXECUTION
            sanctuary.set_motor_state(agent.efficiency_momentum, "somatic");
            let mut sanctuary_feedback: Option<quaternity_organism::InteractionResult> = None;

            if let Some(ref impulse) = emerged_impulse {
                if agent_idx == 0 {
                    motor.execute_impulse(&agent.metabolism, impulse);
                }

                let (dx, dy) = impulse.direction;
                agent.position.0 += dx as f32;
                agent.position.1 += dy as f32;
                wrap_position(&mut agent.position);
                agent.phase = direction_to_phase(dx, dy);

                let coords = position_to_voxel(agent.position.0, agent.position.1);
                sanctuary.set_agent_position((coords.0 * 10, coords.1 * 10, 0));
                let energy = intensity_to_energy(impulse.intensity).min(1.0);
                let result = sanctuary.interact(coords, energy, agent.phase, coherence);

                log::info!(
                    "[A{} ACTION] {:?} dir=({},{}), int={:.2} | R={:.3} Res={:.3} Eff={:.3}",
                    agent_idx, impulse.modality, dx, dy, impulse.intensity,
                    result.resistance, result.resonance, result.efficiency
                );

                if energy > 0.001 {
                    agent.efficiency_momentum = SOMATIC_DECAY * agent.efficiency_momentum
                        + (1.0 - SOMATIC_DECAY) * result.efficiency.clamp(0.0, 1.0);
                    agent.somatic_update_count += 1;
                    let q = (if dx > 0 { 1 } else { 0 }) + (if dy > 0 { 2 } else { 0 });
                    agent.trajectory_tracker.record_interaction(q, result.efficiency, (agent_voxel.0, agent_voxel.1));
                }

                sanctuary_feedback = Some(result);
            }

            // SANCTUARY FEEDBACK → STIMULUS
            if let Some(ref feedback) = sanctuary_feedback {
                if feedback.is_resonant {
                    let salience = (feedback.efficiency as f64).max(0.05);
                    let stress = (1.0 - feedback.efficiency).max(0.0) as f64 * 0.3;

                    let stimulus = Stimulus {
                        id: Uuid::new_v4(),
                        source: StimulusSource::Internal {
                            metabolic_state: MetabolicState {
                                coherence: agent.metabolism.get_coherence(),
                                energy_level: feedback.effective_energy as f64,
                                stress_level: stress,
                            },
                        },
                        urgency: salience.min(1.0),
                        novelty: 0.3,
                        salience: salience.min(1.0),
                        timestamp: Instant::now(),
                        attention_count: 0,
                    };
                    agent.attention_field.add(stimulus);
                    agent.observer.signal_new_input();
                } else {
                    let dissonance_strength = (1.0 - feedback.efficiency).max(0.0) as f64;
                    let stress = dissonance_strength * 0.8;

                    let stimulus = Stimulus {
                        id: Uuid::new_v4(),
                        source: StimulusSource::Internal {
                            metabolic_state: MetabolicState {
                                coherence: agent.metabolism.get_coherence(),
                                energy_level: feedback.effective_energy as f64,
                                stress_level: stress,
                            },
                        },
                        urgency: dissonance_strength.min(1.0),
                        novelty: 0.5,
                        salience: dissonance_strength.min(1.0),
                        timestamp: Instant::now(),
                        attention_count: 0,
                    };
                    agent.attention_field.add(stimulus);
                }
            }
        } // end per-agent loop

        // ═══════════════════════════════════════════════════════════════════
        // SHARED: Record tick samples for ALL agents, then tick the field
        // ═══════════════════════════════════════════════════════════════════
        for (i, ag) in agents.iter().enumerate() {
            sanctuary.set_agent_id(&format!("agent_{}", i));
            let vehicle_str = outcome_to_vehicle(ag.observer.last_outcome.as_ref());
            sanctuary.record_tick_sample(
                ag.position.0 as i32,
                ag.position.1 as i32,
                ag.metabolism.get_coherence(),
                ag.efficiency_momentum,
                "somatic",
                &vehicle_str,
            );
        }
        sanctuary.set_agent_id("agent_0");
        let coherence = agents[0].metabolism.get_coherence();
        sanctuary.tick();

        // ═══════════════════════════════════════════════════════════════════
        // PHASE 6: LEGACY LEARNING (Visual-Audio Correlation)
        // ═══════════════════════════════════════════════════════════════════
        // LOGGING (agent 0 primary, multi-agent summary periodic)
        // ═══════════════════════════════════════════════════════════════════
        let elapsed = agents[0].metabolism.get_elapsed_seconds();
        let avg_efficiency = sanctuary.average_efficiency(10);
        let avg_resonance = sanctuary.average_resonance(10);
        let authorized = authorize_action(coherence);

        let status = if coherence < 0.3 { "CRITICAL" } else if coherence < 0.7 { "UNSTABLE" } else { "STABLE" };
        let motor_status = if authorized { "UNLOCKED" } else { "LOCKED" };
        let obs0 = &agents[0].observer;
        let obs_status = if obs0.state.is_stagnant() { "STAGNANT" } else if obs0.state.readiness > 0.5 { "BUILDING" } else { "SCANNING" };

        println!(
            "[{:.1}s] A0 Coh:{:.2} ({}) | Obs:{} r={:.2} | Sanc: eff={:.2} res={:.2} | {} | {} agents",
            elapsed, coherence, status,
            obs_status, obs0.state.readiness,
            avg_efficiency, avg_resonance,
            motor_status, NUM_AGENTS
        );

        if tick_count % 90 == 0 {
            for (i, ag) in agents.iter().enumerate() {
                let vx = position_to_voxel(ag.position.0, ag.position.1);
                log::info!(
                    "[AGENT {}] pos=({:.0},{:.0}) voxel=({},{}) eff_mom={:.4} somatic_updates={} obs_r={:.2}",
                    i, ag.position.0, ag.position.1, vx.0, vx.1,
                    ag.efficiency_momentum, ag.somatic_update_count, ag.observer.state.readiness
                );
            }
            log::info!(
                "[SANCTUARY] Voxels: {} | Interactions: {} | Avg Efficiency: {:.3} | Avg Resonance: {:.3} | Energy: {:.2} | Hours: {:.4}",
                sanctuary.active_voxel_count(), sanctuary.interaction_count(),
                sanctuary.average_efficiency(90), sanctuary.average_resonance(90),
                sanctuary.total_field_energy(), sanctuary.get_engine_hours()
            );
        }

        #[cfg(feature = "observability")]
        {
            use quaternity_organism::AudioAnalysis;
            let audio_for_obs = Some(AudioAnalysis {
                volume: audio_volume as f64,
                entropy: audio_entropy,
                dominant_freq: 0.0,
                spectrum: vec![],
            });
            if let Err(e) = rt.block_on(observability.index_metabolism_tick(&agents[0].metabolism, authorized, Some(0.0), audio_for_obs)) {
                log::debug!("Failed to index thought: {}", e);
            }
        }

        // Calculate sleep duration to maintain 90Hz
        let elapsed_time = loop_start.elapsed();
        if elapsed_time < target_interval {
            let sleep_duration = target_interval - elapsed_time;
            std::thread::sleep(sleep_duration);
        } else {
            log::warn!(
                "Loop exceeded target interval: {:.2}ms > {:.2}ms",
                elapsed_time.as_secs_f64() * 1000.0,
                target_interval.as_secs_f64() * 1000.0
            );
        }
    }

    // Regeneration (R >= D): Serialize state before shutdown
    log::info!("Serializing state...");
    if let Err(e) = sanctuary.serialize_state() {
        log::error!("Failed to serialize state: {}", e);
    } else {
        log::info!("State preserved. Run ID: {}.", sanctuary.get_run_id());
    }
    
    // Flush Sanctuary buffers (Dual-Stream persistence)
    if let Err(e) = sanctuary.flush_all() {
        log::error!("Failed to flush Sanctuary buffers: {}", e);
    }
    
    log::info!("Entity shutdown complete. Coherence at shutdown: {:.2}", agents[0].metabolism.get_coherence());
    for (i, ag) in agents.iter().enumerate() {
        log::info!("Agent {}: MemoryGraph {} nodes {} links, mystery {:.1}%",
            i, ag.memory_graph.node_count(), ag.memory_graph.link_count(),
            ag.memory_graph.mystery_ratio() * 100.0);
    }
    log::info!("---");
    log::info!("SANCTUARY FINAL METRICS:");
    log::info!("   Total Interactions: {}", sanctuary.interaction_count());
    log::info!("   Active Voxels: {}", sanctuary.active_voxel_count());
    log::info!("   Total Field Energy: {:.2}", sanctuary.total_field_energy());
    log::info!("   Final Avg Efficiency: {:.4}", sanctuary.average_efficiency(100));
    log::info!("   Final Avg Resonance: {:.4}", sanctuary.average_resonance(100));
    log::info!("   Engine Hours: {:.6}", sanctuary.get_engine_hours());
    log::info!("   Total Runtime: {:?}", sanctuary.get_total_runtime());
    log::info!("   Total Metrics Written: {}", sanctuary.get_total_metrics_written());
    log::info!("   Metrics Epochs: {}", sanctuary.get_metrics_epoch());
    log::info!("   Run ID: {}", sanctuary.get_run_id());
    log::info!("   Data written to: data/metrics/ and data/logs/");
}

/// Extract visual signature from a stimulus and frame.
fn extract_visual_signature(
    stimulus: &Stimulus,
    frame: Option<&DynamicImage>,
) -> Option<VisualSignature> {
    if let (StimulusSource::VisualRegion { rect, entropy, average_color }, Some(f)) = (&stimulus.source, frame) {
        let gray: image::GrayImage = f.to_luma8();
        
        let mut shape_features = Vec::new();
        let mut color_features = Vec::new();
        
        let region_gray = extract_region(&gray, &rect);
        if let Some(region) = region_gray {
            let edges = imageproc::edges::canny(&region, 50.0, 100.0);
            let edge_density = edges.pixels().filter(|p| p[0] > 0).count() as f64 / (edges.width() * edges.height()) as f64;
            shape_features.push(edge_density);
            shape_features.push(rect.width as f64 / rect.height as f64);
        }
        
        color_features.push(average_color[0] as f64 / 255.0);
        color_features.push(average_color[1] as f64 / 255.0);
        color_features.push(average_color[2] as f64 / 255.0);
        
        let spatial_features = vec![
            rect.x as f64 / 1920.0,
            rect.y as f64 / 1080.0,
            (rect.width * rect.height) as f64 / (1920.0 * 1080.0),
        ];
        
        Some(VisualSignature {
            shape_features,
            color_features,
            spatial_features,
        })
    } else {
        None
    }
}

/// Extract audio signature from audio analysis.
fn extract_audio_signature_from_analysis(audio: &AudioAnalysis) -> Option<AudioSignature> {
    let freq_profile: Vec<f64> = audio.spectrum.iter()
        .take(20)
        .map(|&mag| {
            let max_mag = audio.spectrum.iter().fold(0.0_f64, |a: f64, b: &f64| a.max(*b));
            if max_mag > 0.0 { mag / max_mag } else { 0.0 }
        })
        .collect();
    
    let temporal_pattern = vec![
        audio.volume,
        audio.entropy,
        1.0 - audio.entropy,  // Harmonic ratio proxy
    ];
    
    Some(AudioSignature {
        frequency_profile: freq_profile,
        temporal_pattern,
        dominant_freq: audio.dominant_freq,
    })
}

/// Extract a region from a grayscale image.
fn extract_region(img: &image::GrayImage, rect: &Rect) -> Option<image::GrayImage> {
    let mut pixels = Vec::new();
    
    for y in rect.y..(rect.y + rect.height).min(img.height()) {
        for x in rect.x..(rect.x + rect.width).min(img.width()) {
            pixels.push(img.get_pixel(x, y)[0]);
        }
    }
    
    if pixels.is_empty() {
        return None;
    }
    
    image::GrayImage::from_raw(rect.width, rect.height, pixels)
}
