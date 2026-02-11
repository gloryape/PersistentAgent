//! 🔺 Triune Processing System
//!
//! The Triune system represents the dual-aspect processing of stimuli:
//! - Analytical Mind (Logos): Structural truth, consistency, expectation matching
//! - Experiential Heart (Resonance): Felt sense, salience, resonance/aversion
//!
//! Together they produce measurements (not opinions) that the Observer uses
//! to detect dissonance and classify outcomes.
//!
//! Key principle: These modules produce MEASUREMENTS, not DECISIONS.
//! They are instruments, not agents.

pub mod analytical;
pub mod experiential;
pub mod processor;

pub use analytical::{AnalyticalMind, AnalyticalAssessment};
pub use experiential::{ExperientialHeart, ExperientialAssessment};
pub use processor::{TriuneProcessor, TriuneResult};

