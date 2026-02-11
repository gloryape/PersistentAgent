//! 🌌 Explorer Vehicle - Possibility / Curiosity Perspective
//!
//! The Explorer holds open what others want to close.
//! It asks: "What happens if we don't resolve this yet?"
//!
//! Questions it answers:
//! - "What happens if I don't resolve this yet?"
//! - "What possibilities exist in this uncertainty?"
//! - "What questions should be asked?"
//! - "What would I learn by waiting?"
//!
//! Strength: Enables learning and emergence
//! Blind spot: Can spiral without containment if isolated
//!
//! CRITICAL: Explorer is gated by Presence.
//! If Explorer is active twice in succession without new external input,
//! Presence must emit a reanchor event. This prevents runaway abstraction.

use super::{Vehicle, VehicleType, Perspective, MemoryContext};
use crate::cognition::triune::TriuneResult;
use crate::motor::Modality;

/// The Explorer Vehicle - Possibility awareness
pub struct ExplorerVehicle {
    /// How much dissonance/novelty triggers exploration
    exploration_threshold: f32,
    /// Counter for consecutive activations (for Presence gating)
    activation_count: u32,
}

impl ExplorerVehicle {
    /// Create a new Explorer vehicle
    pub fn new() -> Self {
        Self {
            exploration_threshold: 0.4,
            activation_count: 0,
        }
    }

    /// Assess possibility space from triune data
    fn assess_possibility_space(&self, triune: &TriuneResult) -> f32 {
        // Possibility space is HIGH when:
        // - Novelty is high (new territory)
        // - Dissonance is present (unresolved tension = potential)
        // - Signal strength is meaningful (not just noise)
        
        let novelty = triune.analytical.structural_novelty;
        let dissonance = triune.dissonance;
        let signal = (triune.analytical.signal_strength + triune.experiential.signal_strength) / 2.0;
        
        // Possibility = novelty * (1 + dissonance) * signal
        // Dissonance amplifies possibility (tension creates potential)
        let raw_possibility = novelty * (1.0 + dissonance) * signal;
        
        // Normalize to 0.0-1.0
        raw_possibility.min(1.0)
    }

    /// Assess confidence (Explorer is confident about uncertainty!)
    fn assess_confidence(&self, triune: &TriuneResult) -> f32 {
        // Explorer is most confident when things are UNCLEAR
        // High coherence = low explorer confidence (nothing to explore)
        // High dissonance = high explorer confidence (much to explore)
        
        let exploration_value = 1.0 - triune.coherence + triune.dissonance * 0.5;
        exploration_value.max(0.0).min(1.0)
    }

    /// Generate interpretation string
    fn generate_interpretation(&self, possibility_space: f32, triune: &TriuneResult) -> String {
        let novelty = triune.analytical.structural_novelty;
        let dissonance = triune.dissonance;
        
        if possibility_space > 0.7 {
            if dissonance > 0.5 {
                "Rich possibility space. The dissonance holds unresolved potential. Hold open.".to_string()
            } else {
                "Novel territory with room to explore. Questions abound.".to_string()
            }
        } else if possibility_space > 0.4 {
            if novelty > 0.5 {
                "Some novelty to explore, but not urgent. Curiosity is optional.".to_string()
            } else {
                "Limited possibility space. This may not need exploration.".to_string()
            }
        } else {
            "Well-charted territory. Little to explore here.".to_string()
        }
    }

    /// Reset activation count (called when new input arrives)
    pub fn reset_activation(&mut self) {
        self.activation_count = 0;
    }

    /// Get current activation count (for Presence to check)
    pub fn get_activation_count(&self) -> u32 {
        self.activation_count
    }
}

impl Vehicle for ExplorerVehicle {
    fn vehicle_type(&self) -> VehicleType {
        VehicleType::Explorer
    }

    fn interpret(&self, triune: &TriuneResult, memory_context: Option<&MemoryContext>) -> Perspective {
        let possibility_space = self.assess_possibility_space(triune);
        let mut confidence = self.assess_confidence(triune);
        
        // Memory context affects exploration
        if let Some(ctx) = memory_context {
            // If we've explored similar territory before, slightly less confident
            // (we've been here before, might not need to explore again)
            if ctx.familiarity > 0.7 {
                confidence = (confidence - 0.1).max(0.0);
            }
            // If past exploration led to good outcomes, more confident
            if ctx.past_outcome_valence > 0.7 {
                confidence = (confidence + 0.1).min(1.0);
            }
        }
        
        // Explorer always suggests waiting/observing (eyes/ears) rather than acting
        // The point is to NOT resolve, to hold open
        let modality_hint = if possibility_space > self.exploration_threshold {
            // High possibility = observe more
            Some(Modality::Eyes)  // Look for more information
        } else {
            // Low possibility = no strong suggestion
            None
        };
        
        Perspective {
            vehicle_type: VehicleType::Explorer,
            structural_truth: None,
            emotional_intent: None,
            identity_alignment: None,
            possibility_space: Some(possibility_space),
            modality_hint,
            confidence,
            interpretation: self.generate_interpretation(possibility_space, triune),
        }
    }
}

impl Default for ExplorerVehicle {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;
    use crate::cognition::triune::{AnalyticalAssessment, ExperientialAssessment, TriuneResult};

    #[test]
    fn test_explorer_high_possibility() {
        let explorer = ExplorerVehicle::new();
        
        // High novelty + high dissonance = high possibility
        let analytical = AnalyticalAssessment {
            internal_consistency: 0.5,
            expectation_match: 0.3,
            structural_novelty: 0.9,
            signal_strength: 0.8,
            timestamp: Instant::now(),
        };
        
        let experiential = ExperientialAssessment {
            resonance: 0.4,
            aversion: 0.6,
            salience: 0.7,
            signal_strength: 0.8,
            timestamp: Instant::now(),
        };
        
        let triune = TriuneResult::from_assessments(analytical, experiential);
        let perspective = explorer.interpret(&triune, None);
        
        assert!(perspective.possibility_space.unwrap() > 0.5);
        assert!(perspective.confidence > 0.4);
    }
}

