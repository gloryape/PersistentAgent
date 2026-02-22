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

use super::{Vehicle, VehicleType, Perspective, MemoryContext, EnvironmentContext};
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

    fn interpret(
        &self,
        triune: &TriuneResult,
        memory_context: Option<&MemoryContext>,
        env_context: Option<&EnvironmentContext>,
    ) -> Perspective {
        let env = match env_context {
            Some(e) => e,
            None => return Perspective::empty(VehicleType::Saitama),
        };

        // Structural truth: which quadrant has the best stiffness-to-amplitude ratio?
        // Low stiffness + moderate amplitude = navigable. High stiffness = stiffened wake = hostile.
        let mut quadrant_scores = [0.0f32; 4];
        for q in 0..4 {
            let stiff = env.quadrant_stiffness[q];
            let amp = env.quadrant_amplitude[q];
            if amp < 0.01 {
                quadrant_scores[q] = 0.5; // Empty: neutral
            } else {
                quadrant_scores[q] = (1.0 - stiff / (stiff + 1.0)).max(0.0);
            }
        }

        let best_q = quadrant_scores
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(i, _)| i)
            .unwrap_or(0);
        let structural_truth = quadrant_scores[best_q];
        let best_q = best_q as u8;

        let mut confidence = triune.analytical.signal_strength;
        if let Some(ctx) = memory_context {
            confidence = (confidence + ctx.familiarity * 0.1).min(1.0);
        }

        let modality_hint = if structural_truth > self.truth_threshold {
            Some(Modality::Movement)
        } else if structural_truth < 0.3 {
            Some(Modality::Rest)
        } else {
            Some(Modality::Eyes)
        };

        Perspective {
            vehicle_type: VehicleType::Saitama,
            structural_truth: Some(structural_truth),
            emotional_intent: None,
            identity_alignment: None,
            possibility_space: None,
            modality_hint,
            recommended_quadrant: Some(best_q),
            confidence,
            interpretation: format!(
                "Structural analysis: Q{} most navigable (score={:.2}, stiff={:.2})",
                best_q, quadrant_scores[best_q as usize], env.quadrant_stiffness[best_q as usize]
            ),
        }
    }
}

impl Default for SaitamaVehicle {
    fn default() -> Self {
        Self::new()
    }
}

