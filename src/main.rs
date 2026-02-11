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
    PreProcessor, Observer, MemoryBank, VisualSignature, AudioSignature,
    Stimulus, StimulusSource, Rect, AudioPattern,
    // New architecture
    AttentionField, MemoryGraph, MotorImpulse,
    // Sanctuary (4D Scalar Field Environment)
    Sanctuary,
    // Bio-Mimetic Sensory System
    BioRetina, BioCochlea, Transducer, SensoryFrame,
};
use quaternity_organism::cognition::{
    TriuneProcessor, VehicleSystem, WitnessOutcome,
    sanctuary::{direction_to_phase, intensity_to_energy, position_to_voxel},
    stimuli::MetabolicState,
};
use serde_json;
use uuid::Uuid;
#[cfg(feature = "observability")]
use quaternity_organism::observability::ThoughtIndexer;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::path::PathBuf;
use std::thread;

use quaternity_organism::checkpoint::{self, save_simulation, OrganismCheckpoint};

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

fn main() {
    // Initialize logger with timestamp formatting
    env_logger::Builder::from_default_env()
        .format_timestamp_secs()
        .init();

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
    
    // Initialize transducer from stdin (SENS protocol)
    let mut transducer = Transducer::from_stdin();
    log::info!("Transducer initialized (reading from stdin)");
    
    // Initialize Retina (Physical Surface)
    let mut bio_retina = BioRetina::default();
    log::info!("Retina initialized: {}x{} receptor grid", 
        bio_retina.resolution().0, bio_retina.resolution().1);
    
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

    let running_save = Arc::clone(&running);
    thread::spawn(move || {
        let request_path = PathBuf::from("data").join("save_request.txt");
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

    // Create Metabolism instance (coherence starts at 0.0)
    let mut metabolism = Metabolism::new();
    let target_interval = Duration::from_nanos(11_111_111); // 11.11ms for 90Hz

    // Initialize Cognition Systems
    let mut preprocessor = PreProcessor::new();
    let mut attention_field = AttentionField::new();
    let mut observer = Observer::new();
    let mut triune = TriuneProcessor::new();
    let mut vehicles = VehicleSystem::new();
    let mut memory_graph = MemoryGraph::new();
    let mut memory_bank = MemoryBank::new();  // Legacy, for correlations

    // Agent position/phase and tick (from Load or defaults)
    let (mut agent_position, mut agent_phase, mut tick_count) = if let Some(ref org) = loaded_organism {
        metabolism.set_coherence(org.coherence);
        (org.agent_position_pixels, org.agent_phase, org.tick_count)
    } else {
        ((960.0, 540.0), 0.0, 0u32) // Center of 1920x1080
    };

    log::info!("Starting 90Hz processing loop...");
    log::info!("Target interval: {:.2}ms (90Hz)", target_interval.as_secs_f64() * 1000.0);
    log::info!("Coherence threshold for motor unlock: 0.70");
    log::info!("Observer Architecture activated");
    log::info!("   - AttentionField for relational context");
    log::info!("   - Triune (Analytical + Experiential)");
    log::info!("   - Vehicles (Saitama, Complement, Identity, Explorer)");
    log::info!("   - MemoryGraph for persistent identity bindings");
    log::info!("   - Sanctuary (4D Scalar Field with Causal Memory)");
    log::info!("---");

    // State tracking
    let mut orient_counter = 0u32;  // Count ticks since last orientation
    let orient_interval = 5u32;     // Re-orient every 5 ticks or on stagnation
    let mut last_frame: Option<SensoryFrame> = None;

    // Motor momentum state — infant babbling isn't purely random,
    // there's autocorrelation in limb movements. This gives the organism
    // a chance to deposit consistent phase at consecutive voxels.
    let mut last_babble_dx: i32 = 0;
    let mut last_babble_dy: i32 = 0;
    const MOTOR_MOMENTUM: f32 = 0.7; // 70% previous direction, 30% random

    // Main 90Hz loop
    loop {
        if !running.load(Ordering::SeqCst) {
            break;
        }

        let loop_start = Instant::now();

        // Perform metabolism tick (measures timing and updates coherence)
        let coherence = metabolism.tick();
        let elapsed = metabolism.get_elapsed_seconds();

        tick_count += 1;
        orient_counter += 1;

        // Handle save request (dashboard wrote data/save_request.txt with name)
        if SAVE_REQUESTED.swap(false, Ordering::SeqCst) {
            let name = SAVE_NAME.get().and_then(|m| m.lock().ok().and_then(|mut g| g.take()))
                .unwrap_or_else(|| format!("autosave_{}", tick_count));
            if let Err(e) = save_simulation(
                &sanctuary,
                agent_position,
                agent_phase,
                metabolism.get_coherence(),
                tick_count,
                &name,
            ) {
                log::error!("Save failed: {}", e);
            }
        }

        // ═══════════════════════════════════════════════════════════════════
        // PHASE 1: SENSORY GATHERING (Direct Injection from stdin)
        // ═══════════════════════════════════════════════════════════════════

        // Read frame from stdin (SENS protocol). If EOF or no data, continue with vacuum fluctuations
        let frame = match transducer.read_frame() {
            Ok(f) => {
                last_frame = Some(f.clone());
                Some(f)
            }
            Err(_) => {
                // EOF or no data available - continue with internal dynamics (vacuum fluctuations)
                // Do not exit; entity continues processing without external input
                last_frame.clone()
            }
        };

        // Calculate visual/audio metrics for logging (initialize defaults)
        let mut visual_entropy = 0.0;
        let mut audio_volume = 0.0;
        let mut audio_entropy = 0.5;

        // ══════════════════════════════════════════════════════════════════
        // STEP 1: External stimuli deform the field (if available)
        // External video/audio still inject into the Sanctuary, but at the
        // field coordinates (0-63, 0-63) — they change the environment.
        // ══════════════════════════════════════════════════════════════════
        if let Some(ref sensory_frame) = frame {
            // Retina: Inject photons as energy (RGB → amplitude + phase)
            bio_retina.inject_into_sanctuary(&sensory_frame.rgb_grid, &mut sanctuary);
            
            // Cochlea: Inject sound pressure as displacement
            bio_cochlea.inject_into_sanctuary(&sensory_frame.audio_samples, &mut sanctuary);
            
            // Audio volume for logging/stress
            audio_volume = sensory_frame.audio_samples.iter()
                .map(|&s| s.abs())
                .sum::<f32>() / sensory_frame.audio_samples.len() as f32;
            
            audio_entropy = if audio_volume > 0.1 {
                0.3 + (audio_volume * 0.7) as f64
            } else {
                0.0
            };
            
            if audio_volume > 0.5 {
                metabolism.apply_audio_stress(audio_volume as f64, audio_entropy, 0.0);
            }
        }

        // ══════════════════════════════════════════════════════════════════
        // STEP 2: PROPRIOCEPTION — The organism perceives its own field
        //
        // Instead of sampling the full 64x64 grid every tick (too expensive),
        // we do lightweight directional probing: sample the field state in
        // the 4 cardinal directions + current position. This gives the
        // organism spatial awareness of its terrain without the cost of
        // 4096 HashMap lookups per tick.
        //
        // Every 15 ticks (matching babble rate), we do the full 64x64 sample
        // and generate a visual stimulus for the cognitive pipeline.
        //
        // The organism READS the field — it does NOT re-inject what it sees.
        // Seeing is perception, not action.
        // ══════════════════════════════════════════════════════════════════
        let agent_voxel = position_to_voxel(agent_position.0, agent_position.1);

        // Lightweight per-tick probe: sample 5 points around the organism
        let probe_radius = 3i32; // 3 voxels ahead in each direction
        let here_amp = sanctuary.density_at((agent_voxel.0, agent_voxel.1, 0));
        let here_phase = sanctuary.phase_at((agent_voxel.0, agent_voxel.1, 0));
        let north_amp = sanctuary.density_at((agent_voxel.0, agent_voxel.1 - probe_radius, 0));
        let south_amp = sanctuary.density_at((agent_voxel.0, agent_voxel.1 + probe_radius, 0));
        let east_amp  = sanctuary.density_at((agent_voxel.0 + probe_radius, agent_voxel.1, 0));
        let west_amp  = sanctuary.density_at((agent_voxel.0 - probe_radius, agent_voxel.1, 0));

        // Compute directional contrast: where is the field brightest/darkest?
        let max_dir_amp = north_amp.max(south_amp).max(east_amp).max(west_amp);
        let min_dir_amp = north_amp.min(south_amp).min(east_amp).min(west_amp);
        let directional_contrast = if max_dir_amp > 0.001 {
            (max_dir_amp - min_dir_amp) / max_dir_amp
        } else {
            0.0
        };

        // Visual entropy from local field state
        let local_amplitudes = [here_amp, north_amp, south_amp, east_amp, west_amp];
        let mean_amp = local_amplitudes.iter().sum::<f32>() / 5.0;
        let variance = local_amplitudes.iter()
            .map(|a| (a - mean_amp).powi(2))
            .sum::<f32>() / 5.0;
        visual_entropy = (variance.sqrt() / 2.0).min(1.0) as f64;

        if visual_entropy > 0.8 {
            metabolism.apply_visual_stress(visual_entropy);
        }

        // Full proprioceptive scan every 15 ticks → generates visual stimulus
        // for the cognitive pipeline. The organism "sees" its environment.
        if tick_count % 15 == 0 {
            let proprioceptive_frame = bio_retina.sample_from_sanctuary(
                &sanctuary,
                (agent_voxel.0, agent_voxel.1),
            );

            // Compute spatial features from the proprioceptive frame
            let mut total_brightness = 0.0f32;
            let mut bright_pixels = 0u32;
            let mut total_pixels = 0u32;
            // Quadrant brightness (what's ahead in each direction)
            let mut quadrant_brightness = [0.0f32; 4]; // N, S, E, W
            let mut quadrant_counts = [0u32; 4];
            let (pw, ph) = bio_retina.resolution();
            let half_w = pw / 2;
            let half_h = ph / 2;

            for (y, row) in proprioceptive_frame.iter().enumerate() {
                for (x, &[r, g, b]) in row.iter().enumerate() {
                    let lum = 0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32;
                    total_brightness += lum;
                    total_pixels += 1;
                    if lum > 10.0 { bright_pixels += 1; }

                    // Assign to quadrant
                    let xi = x as u32;
                    let yi = y as u32;
                    if yi < half_h { quadrant_brightness[0] += lum; quadrant_counts[0] += 1; } // North
                    if yi >= half_h { quadrant_brightness[1] += lum; quadrant_counts[1] += 1; } // South
                    if xi >= half_w { quadrant_brightness[2] += lum; quadrant_counts[2] += 1; } // East
                    if xi < half_w { quadrant_brightness[3] += lum; quadrant_counts[3] += 1; } // West
                }
            }

            let mean_brightness = if total_pixels > 0 { total_brightness / total_pixels as f32 } else { 0.0 };
            let coverage = if total_pixels > 0 { bright_pixels as f32 / total_pixels as f32 } else { 0.0 };

            // Normalize quadrant brightness
            for i in 0..4 {
                if quadrant_counts[i] > 0 {
                    quadrant_brightness[i] /= quadrant_counts[i] as f32;
                }
            }

            // Generate a proprioceptive stimulus if the field has visible structure
            // (coverage > 0 means the organism can see SOMETHING in its environment)
            if coverage > 0.01 || directional_contrast > 0.1 {
                let prop_salience = (coverage * 0.5 + directional_contrast * 0.5) as f64;
                let prop_novelty = directional_contrast as f64; // Contrast = something to orient toward

                let proprioceptive_stimulus = Stimulus {
                    id: Uuid::new_v4(),
                    source: StimulusSource::Internal {
                        metabolic_state: MetabolicState {
                            coherence: metabolism.get_coherence(),
                            energy_level: mean_brightness as f64 / 255.0,
                            stress_level: 0.0, // Proprioception isn't stressful
                        },
                    },
                    urgency: prop_salience.min(1.0),
                    novelty: prop_novelty.min(1.0),
                    salience: prop_salience.min(1.0),
                    timestamp: Instant::now(),
                    attention_count: 0,
                };
                attention_field.add(proprioceptive_stimulus);
                observer.signal_new_input();

                log::debug!(
                    "[PROPRIO] coverage={:.2}, contrast={:.2}, bright=[N:{:.0} S:{:.0} E:{:.0} W:{:.0}]",
                    coverage, directional_contrast,
                    quadrant_brightness[0], quadrant_brightness[1],
                    quadrant_brightness[2], quadrant_brightness[3]
                );
            }
        }
        
        let coherence = metabolism.get_coherence();

        let coherence = metabolism.get_coherence();
        let authorized = authorize_action(coherence);

        // ═══════════════════════════════════════════════════════════════════
        // PHASE 2: PRE-PROCESSING (Feature Extraction)
        // ═══════════════════════════════════════════════════════════════════
        // Note: With Bio-Mimetic injection, sensory data directly affects the
        // Sanctuary field. Pre-processing now focuses on internal state changes
        // rather than external feature extraction.

        let new_stimuli = Vec::new(); // Simplified for now - stimuli emerge from field state

        // Add new stimuli to attention field
        if !new_stimuli.is_empty() {
            attention_field.add_batch(new_stimuli);
            observer.signal_new_input();  // Reset presence tracking
        }

        // ═══════════════════════════════════════════════════════════════════
        // PHASE 3: ORIENTATION (Relational Processing)
        // ═══════════════════════════════════════════════════════════════════

        // Re-orient periodically or when stagnant
        let should_orient = orient_counter >= orient_interval 
            || observer.state.is_stagnant()
            || observer.state.current_focus.is_none();

        if should_orient && !attention_field.is_empty() {
            observer.orient(&mut attention_field, &memory_graph);
            orient_counter = 0;
            
            // Decay memory graph links periodically
            if tick_count % 100 == 0 {
                memory_graph.decay_links();
            }
        }

        // ═══════════════════════════════════════════════════════════════════
        // PHASE 4: ENGAGEMENT (The Physics of Choice)
        // ═══════════════════════════════════════════════════════════════════

        let mut action_emerged = false;
        let mut emerged_impulse: Option<MotorImpulse> = None;

        if authorized && !attention_field.is_empty() {
            // Get highest priority stimulus
            if let Some(target) = attention_field.peek_highest_priority().cloned() {
                // Mark as attending
                attention_field.mark_attending(&target.raw.id);
                
                // Process through Observer (Triune + Vehicles + Threshold)
                if let Some(impulse) = observer.attend(&target, &mut triune, &vehicles, &memory_graph) {
                    action_emerged = true;
                    emerged_impulse = Some(impulse.clone());
                    
                    // Unmark attending and potentially remove
                    attention_field.unmark_attending(&target.raw.id);
                    
                    // Update Sanctuary's vehicle tracking for Stream A metrics
                    if let Some(ref outcome) = observer.last_outcome {
                        match outcome {
                            WitnessOutcome::Coherent(_) => sanctuary.set_vehicle("Coherent"),
                            WitnessOutcome::NeedsPerspective(vehicles_needed) => {
                                let vehicle_names: Vec<_> = vehicles_needed.iter()
                                    .map(|v| format!("{:?}", v))
                                    .collect();
                                sanctuary.set_vehicle(&vehicle_names.join("+"));
                            }
                            WitnessOutcome::DeepMystery(_) => sanctuary.set_vehicle("Mystery"),
                            WitnessOutcome::ReanchorPresence => sanctuary.set_vehicle("Reanchor"),
                        }
                    }
                    
                    // Record to MemoryGraph if coherent
                    if let Some(WitnessOutcome::Coherent(_)) = &observer.last_outcome {
                        memory_graph.record_coherent_event(
                            &target,
                            observer.state.coherence,
                            observer.state.inhibition,  // Use inhibition as proxy for dissonance
                            triune.experiential.recent_trend().0,  // Resonance trend
                            Some(impulse.clone()),
                        );
                        
                        // Log to Stream B (JSONL)
                        sanctuary.log_event(
                            "coherent_action",
                            &format!("Action emerged with coherence {:.2}", observer.state.coherence),
                            serde_json::json!({
                                "readiness": observer.state.readiness,
                                "coherence": observer.state.coherence,
                                "inhibition": observer.state.inhibition,
                                "modality": format!("{:?}", impulse.modality),
                                "intensity": impulse.intensity,
                            }),
                        );
                    }
                    
                    // Record DeepMystery too
                    if let Some(WitnessOutcome::DeepMystery(ref question)) = &observer.last_outcome {
                        memory_graph.record_deep_mystery(
                            &target,
                            observer.state.coherence,
                            observer.state.inhibition,
                            triune.experiential.recent_trend().0,
                        );
                        
                        // Log mystery to Stream B
                        sanctuary.log_event(
                            "deep_mystery",
                            "Encountered unresolvable dissonance",
                            serde_json::json!({
                                "coherence": observer.state.coherence,
                                "inhibition": observer.state.inhibition,
                                "question_context": format!("{:?}", (question.stimulus_id, question.nature.as_str())),
                            }),
                        );
                    }
                    
                    // Remove consumed stimulus
                    attention_field.remove(&target.raw.id);
                } else {
                    attention_field.unmark_attending(&target.raw.id);
                }
            }
        }

        // ═══════════════════════════════════════════════════════════════════
        // PHASE 5: MOTOR EXECUTION + SANCTUARY INTERACTION
        // ═══════════════════════════════════════════════════════════════════
        //
        // MotorImpulse updates agent_position (the organism's body in the field).
        // agent_position is then used to interact with the Sanctuary at that voxel.
        // No physical cursor movement—embodiment is purely in Φ = ρe^{iθ}.

        let mut sanctuary_feedback: Option<quaternity_organism::InteractionResult> = None;

        if let Some(ref impulse) = emerged_impulse {
            motor.execute_impulse(&metabolism, impulse);
            
            // Update agent position based on impulse
            let (dx, dy) = impulse.direction;
            agent_position.0 += dx as f32;
            agent_position.1 += dy as f32;
            
            // Update agent phase (intention direction)
            agent_phase = direction_to_phase(dx, dy);
            
            // Interact with Sanctuary field
            let coords = position_to_voxel(agent_position.0, agent_position.1);
            let energy = intensity_to_energy(impulse.intensity);
            let result = sanctuary.interact(coords, energy, agent_phase, coherence);
            
            log::info!(
                "[ACTION] {:?} impulse: dir=({}, {}), int={:.2} | Field: R={:.3}, Res={:.3}, Eff={:.3}",
                impulse.modality,
                impulse.direction.0,
                impulse.direction.1,
                impulse.intensity,
                result.resistance,
                result.resonance,
                result.efficiency
            );
            
            sanctuary_feedback = Some(result);
        } else if authorized && tick_count % 15 == 0 {
            // Motor babbling with directional momentum
            // Instead of fully random, weight toward continuing the previous direction.
            // This gives the organism a chance to deposit consistent phase at consecutive
            // voxels, building resonance trails instead of poisoning the field with
            // random phases.
            use rand::Rng;
            let mut rng = rand::thread_rng();
            let random_dx = rng.gen_range(-10..=10);
            let random_dy = rng.gen_range(-10..=10);
            
            // Blend: 70% previous direction + 30% random
            let dx = if last_babble_dx == 0 && last_babble_dy == 0 {
                // First babble — fully random
                random_dx
            } else {
                ((MOTOR_MOMENTUM * last_babble_dx as f32) + ((1.0 - MOTOR_MOMENTUM) * random_dx as f32)).round() as i32
            };
            let dy = if last_babble_dx == 0 && last_babble_dy == 0 {
                random_dy
            } else {
                ((MOTOR_MOMENTUM * last_babble_dy as f32) + ((1.0 - MOTOR_MOMENTUM) * random_dy as f32)).round() as i32
            };
            
            // Clamp to valid range
            let dx = dx.clamp(-10, 10);
            let dy = dy.clamp(-10, 10);
            
            // Remember this direction for next babble
            last_babble_dx = dx;
            last_babble_dy = dy;
            
            motor.send_impulse(&metabolism, dx, dy);
            
            // Update position and interact with Sanctuary during babbling too
            agent_position.0 += dx as f32;
            agent_position.1 += dy as f32;
            agent_phase = direction_to_phase(dx, dy);
            
            let coords = position_to_voxel(agent_position.0, agent_position.1);
            let energy = 0.5; // Low energy for babbling
            let result = sanctuary.interact(coords, energy, agent_phase, coherence);
            sanctuary_feedback = Some(result);
        }

        // ═══════════════════════════════════════════════════════════════════
        // PHASE 5.5: SANCTUARY FEEDBACK → STIMULUS GENERATION
        // ═══════════════════════════════════════════════════════════════════

        if let Some(ref feedback) = sanctuary_feedback {
            // ══════════════════════════════════════════════════════════════
            // GRADED FEEDBACK: The organism must feel the difference between
            // efficiency 0.03 and 0.06 before it can learn to seek 0.7.
            // 
            // Old system: binary (resonant+efficiency>0.7 → positive, else
            // efficiency<0.3 → negative). This starved the organism because
            // efficiency was stuck at 0.04, so it ONLY received dissonance.
            //
            // New system: continuous gradient. Every interaction generates
            // feedback with salience proportional to efficiency. Even tiny
            // improvements in efficiency produce a slightly more positive
            // signal, giving the cognitive loop a gradient to climb.
            // ══════════════════════════════════════════════════════════════

            if feedback.is_resonant {
                // POSITIVE: Any resonant interaction creates internal signal
                // Salience scales with efficiency — organism feels the gradient
                let salience = (feedback.efficiency as f64).max(0.05);
                let stress = (1.0 - feedback.efficiency).max(0.0) as f64 * 0.3; // Low stress, proportional

                let resonance_stimulus = Stimulus {
                    id: Uuid::new_v4(),
                    source: StimulusSource::Internal {
                        metabolic_state: MetabolicState {
                            coherence: metabolism.get_coherence(),
                            energy_level: feedback.effective_energy as f64,
                            stress_level: stress,
                        },
                    },
                    urgency: salience.min(1.0),
                    novelty: 0.3, // Resonance is familiar, not novel
                    salience: salience.min(1.0),
                    timestamp: Instant::now(),
                    attention_count: 0,
                };
                attention_field.add(resonance_stimulus);
                observer.signal_new_input();
                
                log::debug!("[SANCTUARY] Resonance signal: eff={:.3}, res={:.3}, salience={:.3}",
                    feedback.efficiency, feedback.resonance, salience);
            } else {
                // NEGATIVE: Dissonant interaction — but graded, not constant
                // Salience inversely proportional to efficiency: worse = louder
                let dissonance_strength = (1.0 - feedback.efficiency).max(0.0) as f64;
                let stress = dissonance_strength * 0.8;

                let dissonance_stimulus = Stimulus {
                    id: Uuid::new_v4(),
                    source: StimulusSource::Internal {
                        metabolic_state: MetabolicState {
                            coherence: metabolism.get_coherence(),
                            energy_level: feedback.effective_energy as f64,
                            stress_level: stress,
                        },
                    },
                    urgency: dissonance_strength.min(1.0),
                    novelty: 0.5, // Dissonance is somewhat novel
                    salience: dissonance_strength.min(1.0),
                    timestamp: Instant::now(),
                    attention_count: 0,
                };
                attention_field.add(dissonance_stimulus);
                
                log::debug!("[SANCTUARY] Dissonance signal: eff={:.3}, res={:.3}, stress={:.3}",
                    feedback.efficiency, feedback.resonance, stress);
            }
        }

        // Record current agent voxel every tick so the dashboard field colors update as the entity moves
        sanctuary.record_tick_sample(agent_position.0 as i32, agent_position.1 as i32, coherence);
        // Tick the Sanctuary (apply entropy decay + periodic metrics flush)
        sanctuary.tick();

        // ═══════════════════════════════════════════════════════════════════
        // PHASE 6: LEGACY LEARNING (Visual-Audio Correlation)
        // ═══════════════════════════════════════════════════════════════════
        // Note: With Bio-Mimetic injection, correlations emerge naturally from
        // field state rather than explicit feature matching. This section is
        // kept for backwards compatibility but may be simplified in future.

        // ═══════════════════════════════════════════════════════════════════
        // LOGGING AND STATUS
        // ═══════════════════════════════════════════════════════════════════

        // Determine status
        let status = if coherence < 0.3 {
            "CRITICAL"
        } else if coherence < 0.7 {
            "UNSTABLE"
        } else {
            "STABLE"
        };

        let motor_status = if authorized { "MOTOR UNLOCKED" } else { "MOTOR LOCKED" };

        let visual_status = if visual_entropy > 0.8 {
            "Overload"
        } else if visual_entropy < 0.2 {
            "Calm"
        } else {
            "Active"
        };

        let audio_status = if audio_volume < 0.1 {
            "SILENT"
        } else if audio_entropy < 0.3 {
            "HARMONIC"
        } else if audio_entropy > 0.8 {
            "NOISE"
        } else {
            "ACTIVE"
        };

        // Observer state summary
        let observer_status = if action_emerged {
            "ACTION!"
        } else if observer.state.is_stagnant() {
            "STAGNANT"
        } else if observer.state.readiness > 0.5 {
            "BUILDING"
        } else {
            "SCANNING"
        };

        // Get Sanctuary metrics
        let avg_efficiency = sanctuary.average_efficiency(10);
        let avg_resonance = sanctuary.average_resonance(10);

        // Log every tick
        println!(
            "[{:.1}s] Coh:{:.2} ({:8}) | Vis:{:.2} ({:7}) | Aud:{:.2}/{:.2} ({:7}) | Obs:{:8} r={:.2} | Sanctuary: eff={:.2} res={:.2} | {}",
            elapsed, coherence, status, 
            visual_entropy, visual_status,
            audio_volume, audio_entropy, audio_status,
            observer_status, observer.state.readiness,
            avg_efficiency, avg_resonance,
            motor_status
        );

        // Periodic detailed log
        if tick_count % 90 == 0 {
            log::info!(
                "[STATUS] AttentionField: {} items | MemoryGraph: {} nodes, {} links | MemoryBank: {} correlations | Mysteries: {:.1}%",
                attention_field.len(),
                memory_graph.node_count(),
                memory_graph.link_count(),
                memory_bank.len(),
                memory_graph.mystery_ratio() * 100.0
            );
            log::info!(
                "[SANCTUARY] Voxels: {} | Interactions: {} | Avg Efficiency: {:.3} | Avg Resonance: {:.3} | Total Field Energy: {:.2} | Engine Hours: {:.4}",
                sanctuary.active_voxel_count(),
                sanctuary.interaction_count(),
                sanctuary.average_efficiency(90),
                sanctuary.average_resonance(90),
                sanctuary.total_field_energy(),
                sanctuary.get_engine_hours()
            );
        }

        // Index thought to Elasticsearch (if observability enabled)
        #[cfg(feature = "observability")]
        {
            // Create simplified audio analysis for observability
            use quaternity_organism::AudioAnalysis;
            let audio_for_obs = Some(AudioAnalysis {
                volume: audio_volume as f64,
                entropy: audio_entropy,
                dominant_freq: 0.0, // Not calculated in direct injection system
                spectrum: vec![],   // Not calculated in direct injection system
            });
            if let Err(e) = rt.block_on(observability.index_metabolism_tick(&metabolism, authorized, Some(visual_entropy), audio_for_obs)) {
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
    
    log::info!("Entity shutdown complete. Coherence at shutdown: {:.2}", metabolism.get_coherence());
    log::info!("MemoryGraph: {} nodes, {} links", memory_graph.node_count(), memory_graph.link_count());
    log::info!("MemoryBank: {} correlations stored", memory_bank.len());
    log::info!("Mystery ratio: {:.1}%", memory_graph.mystery_ratio() * 100.0);
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
