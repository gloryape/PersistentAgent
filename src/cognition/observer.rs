//! 👁️ The Unified Observer - Attention, Witness, Presence
//!
//! The Observer is the system's epistemic conscience. It maintains integrity by:
//! - Managing attention (what gets noticed)
//! - Witnessing meaning (classifying coherence vs dissonance)
//! - Instantiating presence (grounding when cognition destabilizes)
//!
//! The Observer is the LITERAL ACTOR. Action emerges as a state transition
//! when readiness crosses a threshold - it is not selected, it becomes inevitable.
//!
//! Key principle: The Observer keeps the agent REAL, not safe, not correct - REAL.

use uuid::Uuid;
use std::time::Instant;

use crate::Metabolism;
use crate::motor::{MotorImpulse, Modality};
use crate::cognition::attention_field::{AttentionField, StimulusContext};
use crate::cognition::memory_graph::MemoryGraph;
use crate::cognition::triune::{TriuneProcessor, TriuneResult};
use crate::cognition::vehicles::{VehicleSystem, VehicleType, VehicleAlignment, EnvironmentContext};

/// Constants for Observer behavior
pub const ACTION_THRESHOLD: f32 = 0.7;     // Inevitability threshold for action
pub const MAX_STAGNATION: u32 = 10;        // Ticks before re-orient
pub const REFRACTORY_READINESS: f32 = 0.1; // Readiness after action
pub const HYSTERESIS: f32 = 0.05;          // Prevent oscillation

/// Legacy Action enum (for backwards compatibility)
/// New code should use MotorImpulse directly
#[derive(Debug, Clone)]
pub enum Action {
    FocusEyes { target: (u32, u32) },
    FocusEars { frequency: f64 },
    Rest,
}

impl Action {
    /// Convert to MotorImpulse
    pub fn to_impulse(&self) -> MotorImpulse {
        match self {
            Action::FocusEyes { target } => MotorImpulse::eyes(*target, 0.7),
            Action::FocusEars { frequency } => MotorImpulse::ears(*frequency, 0.7),
            Action::Rest => MotorImpulse::rest(),
        }
    }
}

/// Outcome of the Observer-Witness classification
#[derive(Debug, Clone)]
pub enum WitnessOutcome {
    /// Coherence achieved - action can emerge
    Coherent(MotorImpulse),
    /// Dissonance detected - need vehicle perspectives
    NeedsPerspective(Vec<VehicleType>),
    /// Deep mystery - unresolvable, record in memory
    DeepMystery(Question),
    /// Presence violation - need to reanchor to "now"
    ReanchorPresence,
}

/// A question generated from deep mystery
#[derive(Debug, Clone)]
pub struct Question {
    /// The stimulus that triggered the question
    pub stimulus_id: Uuid,
    /// The nature of the mystery
    pub nature: String,
    /// When the question was formed
    pub timestamp: Instant,
}

/// Presence event for runaway detection
#[derive(Debug, Clone)]
pub enum PresenceEvent {
    /// Cognition is grounded - no intervention needed
    Stable,
    /// Need to reanchor to present moment
    ReanchorNow,
    /// Explorer vehicle being gated (repeated activation)
    GateExplorer,
}

/// The Observer's internal state
#[derive(Debug, Clone)]
pub struct ObserverState {
    /// Readiness for action (builds toward threshold)
    pub readiness: f32,
    /// Triune coherence (alignment of mind and heart)
    pub coherence: f32,
    /// Experiential resonance (pull toward action)
    pub resonance: f32,
    /// Inhibition (resistance to action)
    pub inhibition: f32,
    /// Pending vehicle alignment (if perspectives gathered)
    pub pending_alignment: Option<VehicleAlignment>,
    /// Stagnation counter (ticks without readiness change)
    pub stagnation_counter: u32,
    /// Last readiness value (for delta calculation)
    pub last_readiness: f32,
    /// Whether threshold has been crossed
    pub crossed_threshold: bool,
    /// Currently attended stimulus
    pub current_focus: Option<Uuid>,
}

impl ObserverState {
    /// Create a new ObserverState
    pub fn new() -> Self {
        Self {
            readiness: 0.0,
            coherence: 0.5,
            resonance: 0.5,
            inhibition: 0.0,
            pending_alignment: None,
            stagnation_counter: 0,
            last_readiness: 0.0,
            crossed_threshold: false,
            current_focus: None,
        }
    }

    /// Update dynamics based on Triune result and vehicle alignment
    pub fn update_dynamics(&mut self, triune: &TriuneResult, alignment: Option<&VehicleAlignment>) {
        self.coherence = triune.coherence;
        self.resonance = triune.experiential.resonance;
        
        let mut base_inhibition = triune.dissonance * 0.5;
        
        if let Some(align) = alignment {
            // Vehicle consensus REDUCES inhibition — they processed the dissonance.
            // High alignment means the dissonance has been interpreted and resolved.
            let resolution_factor = align.alignment_score;
            base_inhibition *= 1.0 - resolution_factor * 0.7;
            
            // Genuine disagreement still adds inhibition
            if !align.dissenting.is_empty() {
                base_inhibition += 0.1 * align.dissenting.len() as f32;
            }
        }
        
        self.inhibition = base_inhibition.min(1.0);
        
        // Calculate readiness
        // Readiness = coherence * resonance * (1 + alignment_boost) - inhibition
        let alignment_boost = alignment
            .map(|a| a.alignment_score * 0.3)
            .unwrap_or(0.0);
        
        let raw_readiness = self.coherence * self.resonance * (1.0 + alignment_boost) - self.inhibition;
        self.readiness = raw_readiness.max(0.0).min(1.0);
        
        // Track stagnation
        let delta = (self.readiness - self.last_readiness).abs();
        if delta < 0.01 {
            self.stagnation_counter += 1;
        } else {
            self.stagnation_counter = 0;
        }
        self.last_readiness = self.readiness;
    }

    /// Check if action is inevitable (threshold crossed).
    /// Stimulus intensity (max_novelty) modulates threshold: as novelty approaches 1.0,
    /// the organism cannot ignore the unknown and action becomes inevitable.
    /// Math: effective_threshold = BASE_THRESHOLD * (1.0 - max_novelty)
    pub fn is_inevitable(&self, max_novelty: f32) -> bool {
        let novelty = max_novelty.clamp(0.0, 1.0);
        let effective_threshold = ACTION_THRESHOLD * (1.0 - novelty);
        // Hysteresis: once crossed, stay crossed until much lower
        if self.crossed_threshold {
            self.readiness > effective_threshold - HYSTERESIS
        } else {
            self.readiness > effective_threshold
        }
    }

    /// Crystallize the current state into a MotorImpulse
    pub fn crystallize_impulse(&self, context: &StimulusContext) -> MotorImpulse {
        use crate::cognition::stimuli::StimulusSource;
        
        let modality = if let Some(ref align) = self.pending_alignment {
            align.suggested_modality.unwrap_or(Modality::Eyes)
        } else {
            match &context.raw.source {
                StimulusSource::VisualRegion { .. } => Modality::Eyes,
                StimulusSource::AudioStream { .. } => Modality::Ears,
                StimulusSource::Proprioceptive { .. } => Modality::Eyes,
                StimulusSource::Internal { .. } => Modality::Rest,
            }
        };
        
        let direction = if let Some(ref align) = self.pending_alignment {
            if let Some(dir) = align.recommended_direction {
                dir
            } else {
                self.default_direction(context)
            }
        } else {
            self.default_direction(context)
        };
        
        MotorImpulse::new(direction, self.readiness, modality)
    }

    /// Default direction when vehicles have no recommendation (e.g. from stimulus source)
    fn default_direction(&self, context: &StimulusContext) -> (i32, i32) {
        use crate::cognition::stimuli::StimulusSource;
        match &context.raw.source {
            StimulusSource::VisualRegion { rect, .. } => {
                let center = rect.center();
                (center.0 as i32, center.1 as i32)
            }
            StimulusSource::AudioStream { frequency, .. } => (*frequency as i32, 0),
            StimulusSource::Proprioceptive { brightest_quadrant, .. } => match brightest_quadrant {
                0 => (-1, -1),
                1 => (1, -1),
                2 => (-1, 1),
                3 => (1, 1),
                _ => (0, 0),
            },
            _ => (0, 0),
        }
    }

    /// Reset after action (refractory period)
    pub fn reset_after_fire(&mut self) {
        self.readiness = REFRACTORY_READINESS;
        self.crossed_threshold = false;
        self.stagnation_counter = 0;
        self.pending_alignment = None;
        self.current_focus = None;
    }

    /// Check if stagnant (should re-orient)
    pub fn is_stagnant(&self) -> bool {
        self.stagnation_counter >= MAX_STAGNATION
    }
}

impl Default for ObserverState {
    fn default() -> Self {
        Self::new()
    }
}

/// Cognitive state for presence checking
pub struct CognitiveState {
    /// Recent witness outcomes
    pub recent_outcomes: Vec<WitnessOutcome>,
    /// Number of vehicle consultations this input cycle
    pub vehicle_consultations: u32,
    /// Whether we've had new external input
    pub has_new_input: bool,
    /// Depth of internal processing
    pub internal_depth: u32,
    /// Explorer activations without new input
    pub explorer_activations: u32,
    /// Ticks since vehicles were first consulted this cycle
    pub ticks_since_consultation: u32,
    /// Whether vehicles have been consulted at least once this cycle
    pub vehicles_active: bool,
}

impl CognitiveState {
    pub fn new() -> Self {
        Self {
            recent_outcomes: Vec::new(),
            vehicle_consultations: 0,
            has_new_input: true,
            internal_depth: 0,
            explorer_activations: 0,
            ticks_since_consultation: 0,
            vehicles_active: false,
        }
    }

    pub fn reset_for_new_input(&mut self) {
        self.has_new_input = true;
        self.internal_depth = 0;
        self.explorer_activations = 0;
        self.vehicle_consultations = 0;
        self.vehicles_active = false;
        self.ticks_since_consultation = 0;
    }

    pub fn record_outcome(&mut self, outcome: WitnessOutcome) {
        let is_needs_perspective = matches!(outcome, WitnessOutcome::NeedsPerspective(_));
        
        self.recent_outcomes.push(outcome);
        if self.recent_outcomes.len() > 10 {
            self.recent_outcomes.remove(0);
        }
        
        // Track consultation window
        if is_needs_perspective {
            if !self.vehicles_active {
                self.vehicles_active = true;
                self.ticks_since_consultation = 0;
            } else {
                self.ticks_since_consultation += 1;
            }
        } else {
            self.vehicles_active = false;
            self.ticks_since_consultation = 0;
        }
        
        self.has_new_input = false;
    }
}

impl Default for CognitiveState {
    fn default() -> Self {
        Self::new()
    }
}

/// The Unified Observer
pub struct Observer {
    /// Observer's internal state
    pub state: ObserverState,
    /// Cognitive state for presence tracking
    pub cognitive_state: CognitiveState,
    /// Last witness outcome
    pub last_outcome: Option<WitnessOutcome>,
    /// Legacy fields for backwards compatibility
    pub current_attention: Option<Uuid>,
    pub curiosity_level: f64,
    pub learning_mode: bool,
}

impl Observer {
    /// Create a new Observer
    pub fn new() -> Self {
        Self {
            state: ObserverState::new(),
            cognitive_state: CognitiveState::new(),
            last_outcome: None,
            current_attention: None,
            curiosity_level: 0.8,
            learning_mode: true,
        }
    }

    // ═══════════════════════════════════════════════════════════════════
    // OBSERVER-ATTENTION: Perceptual Gate
    // ═══════════════════════════════════════════════════════════════════

    /// Orient phase: Scan field, enrich with memory, calculate contextual salience
    ///
    /// This is the "look around and see what matters" phase.
    /// Called when:
    /// - New input arrives
    /// - After action (immediate re-orient)
    /// - After stagnation (try different focus)
    pub fn orient(&mut self, field: &mut AttentionField, memory: &MemoryGraph) {
        // Reset cognitive state for new orientation
        self.cognitive_state.has_new_input = true;
        
        // 1. Decay existing salience
        field.decay_salience();
        
        // 2. Find spatial/temporal relations between stimuli
        field.find_spatial_relations();
        field.find_temporal_relations();
        
        // 3. Enrich each stimulus with memory associations
        let ids: Vec<Uuid> = field.ids();
        for id in ids {
            if let Some(context) = field.get_mut(&id) {
                // Query memory for echoes
                let echoes = memory.find_associations(context);
                
                // Add memory echoes (updates contextual salience)
                for echo in echoes {
                    context.add_memory_echo(echo);
                }
            }
        }
        
        // 4. Re-prioritize field by contextual salience
        field.prioritize();
        
        log::debug!("[OBSERVER] Oriented: {} stimuli in field", field.len());
    }

    // ═══════════════════════════════════════════════════════════════════
    // OBSERVER-WITNESS: Meaning Gate
    // ═══════════════════════════════════════════════════════════════════

    /// Attend phase: Engage with selected stimulus, process through Triune,
    /// classify outcome, potentially trigger action.
    ///
    /// This is the "lock on and see if I move" phase.
    /// Returns Some(MotorImpulse) if action emerges, None otherwise.
    pub fn attend(
        &mut self,
        target: &StimulusContext,
        triune: &mut TriuneProcessor,
        vehicles: &VehicleSystem,
        memory: &MemoryGraph,
        env_context: Option<&EnvironmentContext>,
        _efficiency: f32,  // retained for API compatibility; inevitability now uses stimulus (novelty)
    ) -> Option<MotorImpulse> {
        // Record focus
        self.state.current_focus = Some(target.raw.id);
        self.current_attention = Some(target.raw.id);
        
        // 1. Process through Triune
        let triune_result = triune.process_context(target);
        
        log::debug!(
            "[OBSERVER] Triune: coherence={:.2}, dissonance={:.2}",
            triune_result.coherence,
            triune_result.dissonance
        );
        
        // 2. Classify outcome
        let outcome = self.witness(&triune_result, vehicles, memory, target, env_context);
        self.last_outcome = Some(outcome.clone());
        self.cognitive_state.record_outcome(outcome.clone());
        
        // 3. Handle outcome
        match outcome {
            WitnessOutcome::Coherent(impulse) => {
                // Action emerges!
                self.state.crossed_threshold = true;
                self.state.reset_after_fire();
                log::info!("[OBSERVER] Action emerged: {:?}", impulse.modality);
                Some(impulse)
            }
            WitnessOutcome::NeedsPerspective(vehicle_types) => {
                // Consult vehicles and try again
                self.cognitive_state.vehicle_consultations += 1;
                let memory_context = memory.get_memory_context(target);
                let alignment = vehicles.converge(&vehicle_types, &triune_result, Some(&memory_context), env_context);
                
                log::debug!(
                    "[OBSERVER] Consulted {} vehicles, alignment={:.2}",
                    vehicle_types.len(),
                    alignment.alignment_score
                );
                
                // Update state with alignment
                self.state.pending_alignment = Some(alignment.clone());
                self.state.update_dynamics(&triune_result, Some(&alignment));
                
                // Check if now inevitable (stimulus intensity: high novelty forces action)
                let max_novelty = env_context
                    .map(|e| e.quadrant_novelty.iter().cloned().fold(0.0f32, f32::max))
                    .unwrap_or(0.0);
                if self.state.is_inevitable(max_novelty) {
                    let impulse = self.state.crystallize_impulse(target);
                    self.state.reset_after_fire();
                    log::info!("[OBSERVER] Action emerged after vehicle consultation: {:?}", impulse.modality);
                    Some(impulse)
                } else {
                    None
                }
            }
            WitnessOutcome::DeepMystery(question) => {
                log::info!("[OBSERVER] Deep Mystery: {}", question.nature);
                // No action, but this is meaningful
                None
            }
            WitnessOutcome::ReanchorPresence => {
                log::warn!("[OBSERVER] Presence reanchor triggered");
                // Reset and return rest impulse
                self.state.reset_after_fire();
                Some(MotorImpulse::rest())
            }
        }
    }

    /// Witness: Classify using physical field metrics only (thermodynamic phase transitions).
    /// Vehicle selection is decoupled from triune.has_dissonance(); runs unconditionally when env_context available.
    fn witness(
        &mut self,
        triune: &TriuneResult,
        vehicles: &VehicleSystem,
        memory: &MemoryGraph,
        _context: &StimulusContext,
        env_context: Option<&EnvironmentContext>,
    ) -> WitnessOutcome {
        let presence = self.check_presence();
        if let PresenceEvent::ReanchorNow = presence {
            return WitnessOutcome::ReanchorPresence;
        }
        if let PresenceEvent::GateExplorer = presence {
            self.cognitive_state.explorer_activations = 0;
        }

        // No physical data -> stand still until efficiency drops and Explorer triggers
        let env = match env_context {
            Some(e) => e,
            None => return WitnessOutcome::ReanchorPresence,
        };

        // Physics-based selection (thermodynamic phase boundaries)
        let needed_vehicles = vehicles.select_vehicles(triune, Some(env), memory);

        let gated_vehicles: Vec<VehicleType> = if let PresenceEvent::GateExplorer = presence {
            needed_vehicles
                .into_iter()
                .filter(|v| *v != VehicleType::Explorer)
                .collect()
        } else {
            if needed_vehicles.contains(&VehicleType::Explorer) {
                self.cognitive_state.explorer_activations += 1;
            }
            needed_vehicles
        };

        if gated_vehicles.is_empty() {
            return WitnessOutcome::ReanchorPresence;
        }

        WitnessOutcome::NeedsPerspective(gated_vehicles)
    }

    // ═══════════════════════════════════════════════════════════════════
    // OBSERVER-PRESENCE: Meta-Stability Constraint
    // ═══════════════════════════════════════════════════════════════════

    /// Check for runaway cognition and presence violations
    ///
    /// Detects:
    /// - No delta between successive outcomes
    /// - Increasing internal complexity without new input
    /// - Repeated Explorer activation without resolution
    pub fn check_presence(&self) -> PresenceEvent {
        // Check for runaway Explorer
        if self.cognitive_state.explorer_activations >= 2 && !self.cognitive_state.has_new_input {
            log::warn!("[PRESENCE] Explorer gated: {} activations without new input", 
                self.cognitive_state.explorer_activations);
            return PresenceEvent::GateExplorer;
        }
        
        // Vehicle consultation window: allow 3 ticks for readiness to build.
        // Only reanchor if vehicles have been active for too long without resolution.
        if self.cognitive_state.vehicles_active && self.cognitive_state.ticks_since_consultation > 3 {
            log::warn!("[PRESENCE] Reanchor: Vehicle consultation exceeded window ({} ticks)",
                self.cognitive_state.ticks_since_consultation);
            return PresenceEvent::ReanchorNow;
        }
        
        // Too many individual consultations within one input cycle
        if self.cognitive_state.vehicle_consultations >= 5 && !self.cognitive_state.has_new_input {
            log::warn!("[PRESENCE] Reanchor: {} consultations without new input",
                self.cognitive_state.vehicle_consultations);
            return PresenceEvent::ReanchorNow;
        }
        
        // Check for high internal depth without progress
        if self.cognitive_state.internal_depth > 5 && self.state.is_stagnant() {
            log::warn!("[PRESENCE] Reanchor: Deep internal processing without progress");
            return PresenceEvent::ReanchorNow;
        }
        
        PresenceEvent::Stable
    }

    // ═══════════════════════════════════════════════════════════════════
    // LEGACY INTERFACE (for backwards compatibility)
    // ═══════════════════════════════════════════════════════════════════

    /// Legacy: Process queue and generate action
    /// This is a simplified version for backwards compatibility
    pub fn process_queue(
        &mut self,
        queue: &mut crate::cognition::StimuliQueue,
        metabolism: &Metabolism,
    ) -> Option<Action> {
        use crate::cognition::stimuli::StimulusSource;
        
        if metabolism.get_coherence() < 0.7 {
            return Some(Action::Rest);
        }

        let target = queue.peek()?;
        let target_id = target.id;
        let salience = target.salience;
        let urgency = target.urgency;

        if let StimulusSource::VisualRegion { rect, entropy, .. } = &target.source {
            if salience >= (1.0 - self.curiosity_level) {
                let center = rect.center();
                let ent = *entropy;
                queue.mark_attended(target_id);
                self.current_attention = Some(target_id);

                let is_tv = urgency > 0.5 && ent > 0.3;
                if is_tv {
                    log::info!("[NURSERY] Observer focusing on TV center: ({}, {})", center.0, center.1);
                }

                return Some(Action::FocusEyes { target: center });
            }
        }

        if let StimulusSource::AudioStream { frequency, .. } = &target.source {
            if salience >= (1.0 - self.curiosity_level) {
                let freq = *frequency;
                queue.mark_attended(target_id);
                self.current_attention = Some(target_id);

                return Some(Action::FocusEars { frequency: freq });
            }
        }

        if let StimulusSource::Internal { .. } = &target.source {
            queue.mark_attended(target_id);
            self.current_attention = Some(target_id);
            return Some(Action::Rest);
        }

        if let StimulusSource::Proprioceptive { .. } = &target.source {
            // Legacy path: treat proprioceptive like internal (rest).
            // The real processing happens through the witness() pipeline.
            queue.mark_attended(target_id);
            self.current_attention = Some(target_id);
            return Some(Action::Rest);
        }

        None
    }

    /// Legacy: Get current attention
    pub fn get_current_attention(&self) -> Option<Uuid> {
        self.current_attention
    }

    /// Legacy: Set curiosity
    pub fn set_curiosity(&mut self, level: f64) {
        self.curiosity_level = level.clamp(0.0, 1.0);
    }

    /// Legacy: Get curiosity
    pub fn get_curiosity(&self) -> f64 {
        self.curiosity_level
    }

    /// Legacy: Set learning mode
    pub fn set_learning_mode(&mut self, enabled: bool) {
        self.learning_mode = enabled;
        if enabled {
            self.curiosity_level = (self.curiosity_level * 1.2).min(1.0);
        }
    }

    /// Signal new external input (resets presence tracking)
    pub fn signal_new_input(&mut self) {
        self.cognitive_state.reset_for_new_input();
    }
}

impl Default for Observer {
    fn default() -> Self {
        Self::new()
    }
}
