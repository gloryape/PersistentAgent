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

use super::{Vehicle, VehicleType, Perspective, MemoryContext, EnvironmentContext};
use crate::cognition::triune::TriuneResult;
use crate::motor::Modality;
use std::cmp::Ordering;

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

    fn interpret(
        &self,
        triune: &TriuneResult,
        memory_context: Option<&MemoryContext>,
        env_context: Option<&EnvironmentContext>,
    ) -> Perspective {
        let env = match env_context {
            Some(e) => e,
            None => return Perspective::empty(VehicleType::Explorer),
        };

        // Subjective novelty (Memory is Being): steer toward high-novelty quadrants,
        // explicitly AVOIDING lethal max-stiffness walls AND the comfort-trap of the pure vacuum.
        let max_stiff = env.quadrant_stiffness.iter().cloned().fold(0.0f32, f32::max);
        let min_stiff = env.quadrant_stiffness.iter().cloned().fold(f32::MAX, f32::min);
        let at_max_count = env.quadrant_stiffness
            .iter()
            .filter(|&&s| s >= max_stiff * 0.95)
            .count();

        let score = |q: usize| -> f32 {
            let novelty = env.quadrant_novelty[q];
            let stiff = env.quadrant_stiffness[q];
            // Only exclude as lethal when this quadrant is distinctly the stiffest (a wall)
            let lethal = if max_stiff > 0.5 && stiff >= max_stiff * 0.95 && at_max_count < 4 {
                0.0
            } else {
                1.0
            };
            // Exclude vacuum trap (min stiffness = empty corner)
            let vacuum_trap = if min_stiff < f32::MAX && stiff <= min_stiff + 0.001 && min_stiff < 0.01
            {
                0.3
            } else {
                1.0
            };
            novelty * lethal * vacuum_trap
        };

        let (best_q, best_score) = (0..4)
            .map(|q| (q, score(q)))
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(Ordering::Equal))
            .unwrap_or((0, 0.0));

        let max_novelty = env.quadrant_novelty.iter().cloned().fold(0.0f32, f32::max);
        let min_novelty = env.quadrant_novelty.iter().cloned().fold(f32::MAX, f32::min);
        let novelty_gradient = if max_novelty - min_novelty > 0.001 {
            (max_novelty - min_novelty).min(1.0)
        } else {
            0.0
        };
        let confidence = novelty_gradient;

        Perspective {
            vehicle_type: VehicleType::Explorer,
            structural_truth: None,
            emotional_intent: None,
            identity_alignment: None,
            possibility_space: Some(env.quadrant_novelty[best_q]),
            modality_hint: Some(Modality::Eyes),
            recommended_quadrant: Some(best_q as u8),
            confidence,
            interpretation: format!(
                "Subjective novelty: Q{} highest (novelty={:.2}, score={:.2})",
                best_q,
                env.quadrant_novelty[best_q],
                best_score
            ),
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
    fn test_explorer_empty_without_env_context() {
        let explorer = ExplorerVehicle::new();
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
        let perspective = explorer.interpret(&triune, None, None);
        assert!(perspective.recommended_quadrant.is_none());
        assert!(perspective.confidence == 0.0);
    }

    #[test]
    fn test_explorer_high_possibility_with_env_context() {
        let explorer = ExplorerVehicle::new();
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
        let env = EnvironmentContext {
            quadrant_stiffness: [0.5; 4],
            quadrant_amplitude: [0.2, 0.8, 0.1, 0.1],
            quadrant_visit_count: [0, 10, 5, 3],
            quadrant_recent_efficiency: [None; 4],
            current_heading_quadrant: 0,
            heading_consistency: 0.5,
            brightest_quadrant: 1,
            has_novel_structure: [true, false, false, false],
            quadrant_novelty: [0.8, 0.2, 0.3, 0.3],
            agent_voxel: (0, 0, 0),
            has_local_phase_history: [false; 4],
        };
        let perspective = explorer.interpret(&triune, None, Some(&env));
        assert!(perspective.possibility_space.unwrap() > 0.5);
        assert!(perspective.confidence > 0.4);
        assert_eq!(perspective.recommended_quadrant, Some(0));
    }
}

