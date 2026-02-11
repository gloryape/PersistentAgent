//! 🧊 Saitama Vehicle - Structural / Logical Perspective
//!
//! Named after the character who sees through complexity to simple truth.
//!
//! Questions it answers:
//! - "What is the structural truth here?"
//! - "Is this internally consistent?"
//! - "What are the logical constraints?"
//! - "What are the failure modes?"
//!
//! Strength: Prevents nonsense
//! Blind spot: Can justify harm if isolated (cold logic without care)

use super::{Vehicle, VehicleType, Perspective, MemoryContext};
use crate::cognition::triune::TriuneResult;
use crate::motor::Modality;

/// The Saitama Vehicle - Pure logical analysis
pub struct SaitamaVehicle {
    /// Threshold for considering something "true"
    truth_threshold: f32,
}

impl SaitamaVehicle {
    /// Create a new Saitama vehicle
    pub fn new() -> Self {
        Self {
            truth_threshold: 0.6,
        }
    }

    /// Assess structural truth from analytical data
    fn assess_structural_truth(&self, triune: &TriuneResult) -> f32 {
        // Structural truth = internal consistency + expectation match
        let consistency = triune.analytical.internal_consistency;
        let expectation = triune.analytical.expectation_match;
        
        // Weight consistency more heavily (structure over prediction)
        (consistency * 0.6 + expectation * 0.4).min(1.0)
    }

    /// Determine confidence based on signal strength
    fn assess_confidence(&self, triune: &TriuneResult) -> f32 {
        // Confidence is based on how clear the analytical signal is
        triune.analytical.signal_strength * triune.analytical.coherence_score()
    }

    /// Generate interpretation string
    fn generate_interpretation(&self, structural_truth: f32, triune: &TriuneResult) -> String {
        if structural_truth > 0.8 {
            "Structurally consistent and expected. This is likely true.".to_string()
        } else if structural_truth > 0.6 {
            "Moderately consistent. Some uncertainty remains.".to_string()
        } else if structural_truth > 0.4 {
            format!(
                "Inconsistent (consistency={:.2}) or unexpected (match={:.2}). Caution advised.",
                triune.analytical.internal_consistency,
                triune.analytical.expectation_match
            )
        } else {
            "Structurally incoherent. This does not compute.".to_string()
        }
    }
}

impl Vehicle for SaitamaVehicle {
    fn vehicle_type(&self) -> VehicleType {
        VehicleType::Saitama
    }

    fn interpret(&self, triune: &TriuneResult, memory_context: Option<&MemoryContext>) -> Perspective {
        let structural_truth = self.assess_structural_truth(triune);
        let mut confidence = self.assess_confidence(triune);
        
        // Memory context can adjust confidence
        if let Some(ctx) = memory_context {
            // If we've seen similar patterns before, more confidence
            confidence = (confidence + ctx.familiarity * 0.1).min(1.0);
        }
        
        // Determine modality hint based on truth assessment
        let modality_hint = if structural_truth > self.truth_threshold {
            // Truth detected -> proceed with action
            Some(Modality::Movement)
        } else if structural_truth < 0.3 {
            // No truth -> rest/wait for more data
            Some(Modality::Rest)
        } else {
            // Uncertain -> look more closely
            Some(Modality::Eyes)
        };
        
        Perspective {
            vehicle_type: VehicleType::Saitama,
            structural_truth: Some(structural_truth),
            emotional_intent: None,  // Not our domain
            identity_alignment: None,
            possibility_space: None,
            modality_hint,
            confidence,
            interpretation: self.generate_interpretation(structural_truth, triune),
        }
    }
}

impl Default for SaitamaVehicle {
    fn default() -> Self {
        Self::new()
    }
}

