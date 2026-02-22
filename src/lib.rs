//! Quaternity Organism Library
//!
//! Core library module for the Quaternity bio-digital organism.
//!
//! The organism is embodied in the Sanctuary scalar field, with its position
//! at agent_position: (f32, f32) representing its presence in Φ = ρe^{iθ}.
//! The field visualization is its true environment, not desktop control.
//!
//! Provides the authorization gate (Veto system) and module organization.

pub mod metabolism;
pub mod senses;
pub mod motor;
pub mod cognition;
pub mod checkpoint;

#[cfg(feature = "observability")]
pub mod observability;

// Re-export key types
pub use metabolism::Metabolism;
pub use senses::{BioRetina, BioCochlea, Transducer, SensoryFrame};
// Legacy audio analysis type (for observability compatibility)
pub use senses::hearing::AudioAnalysis;
pub use motor::{MotorCortex, MotorImpulse, Modality};
pub use cognition::{
    // Legacy types
    PreProcessor, StimuliQueue, Observer, Action, 
    MemoryBank, VisualSignature, AudioSignature, MemoryReference,
    Stimulus, StimulusSource, Rect, AudioPattern,
    // New architecture types
    AttentionField, StimulusContext, MemoryEcho, StimulusRelation, RelationType,
    TriuneProcessor, TriuneResult, AnalyticalMind, AnalyticalAssessment, 
    ExperientialHeart, ExperientialAssessment,
    VehicleSystem, VehicleType, Perspective, VehicleAlignment, MemoryContext, EnvironmentContext,
    MemoryGraph, MemoryNode, MemorySignature, ResonanceLink, ResonanceType, ConsentResult,
    WitnessOutcome, ObserverState, Question, PresenceEvent, CognitiveState,
    // Sanctuary (4D Scalar Field Environment with Dual-Stream Persistence)
    Sanctuary, FieldState, Voxel, InteractionResult, SanctuaryMetrics,
    MetricRecord, LogRecord,
};

/// Authorization gate - The Veto system.
///
/// Returns `true` if an action is authorized (coherence >= 0.7),
/// `false` if the action should be blocked (coherence < 0.7).
///
/// This is the "Panic/Coma Response" - when coherence is too low,
/// all motor and LLM functions are locked.
pub fn authorize_action(coherence: f64) -> bool {
    coherence >= 0.7
}

