//! 🧠 Cognition Module - Volitional Learning
//!
//! Contains the systems that transform the reflexive organism into a volitional learner:
//! - Stimuli Queue: Organizes raw perception into objects of attention
//! - AttentionField: Relational attention system (enhanced queue)
//! - Pre-Processor: Extracts features from sensory data
//! - Observer: Three roles (Attention, Witness, Presence) as literal actor
//! - Memory: Stores learned correlations
//! - Triune: Analytical Mind + Experiential Heart
//! - Vehicles: Four perspective lenses

pub mod stimuli;
pub mod attention_field;
pub mod preprocessor;
pub mod observer;
pub mod memory;
pub mod triune;
pub mod vehicles;
pub mod memory_graph;
pub mod sanctuary;

// Re-export key types
pub use stimuli::{Stimulus, StimulusSource, StimuliQueue, Rect, AudioPattern, MetabolicState};
pub use attention_field::{AttentionField, StimulusContext, MemoryEcho, StimulusRelation, RelationType};
pub use preprocessor::PreProcessor;
pub use observer::{Observer, Action, WitnessOutcome, ObserverState, Question, PresenceEvent, CognitiveState};
pub use memory::{MemoryBank, MemoryCrystal, VisualSignature, AudioSignature, MemoryReference};
pub use triune::{TriuneProcessor, TriuneResult, AnalyticalMind, AnalyticalAssessment, ExperientialHeart, ExperientialAssessment};
pub use vehicles::{VehicleSystem, VehicleType, Vehicle, Perspective, VehicleAlignment, MemoryContext, EnvironmentContext};
pub use memory_graph::{MemoryGraph, MemoryNode, MemorySignature, ResonanceLink, ResonanceType, ConsentResult};
pub use sanctuary::{
    Sanctuary, SanctuaryCheckpoint, FieldState, Voxel, InteractionResult, SanctuaryMetrics,
    MetricRecord, LogRecord,
    ensure_data_directories, position_to_voxel, direction_to_phase, intensity_to_energy,
    METRICS_BUFFER_SIZE, LOGS_BUFFER_SIZE,
    TerrainFunction, FlatVacuum, PhaseGradient,
    DomainWall, DomainMaze, Vortex, VortexField, CheckerboardDomains,
};

