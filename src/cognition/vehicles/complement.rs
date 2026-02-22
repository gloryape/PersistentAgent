//! ❤️ Complement Vehicle - Relational / Contextual Perspective
//!
//! The Complement sees the relational web, the emotional impact,
//! and the trust dynamics of a situation.
//!
//! Questions it answers:
//! - "What is the emotional intent here?"
//! - "How does this affect others and relationships?"
//! - "What is the social/relational resonance?"
//! - "What trust dynamics are at play?"
//!
//! Strength: Prevents cruelty
//! Blind spot: Can rationalize self-erasure if isolated (putting others first always)

use super::{Vehicle, VehicleType, Perspective, MemoryContext, EnvironmentContext};
use crate::cognition::triune::TriuneResult;
use crate::motor::Modality;

/// The Complement Vehicle - Relational awareness
pub struct ComplementVehicle {
    /// Threshold for positive emotional intent
    positive_threshold: f32,
}

impl ComplementVehicle {
    /// Create a new Complement vehicle
    pub fn new() -> Self {
        Self {
            positive_threshold: 0.5,
        }
    }

    /// Assess emotional intent from experiential data
    fn assess_emotional_intent(&self, triune: &TriuneResult) -> f32 {
        // Emotional intent is about the relational quality
        // High resonance + low aversion = positive intent
        // Low resonance + high aversion = negative intent
        
        let resonance = triune.experiential.resonance;
        let aversion = triune.experiential.aversion;
        
        // Calculate net emotional quality (normalized to 0.0-1.0)
        let raw_intent = resonance - aversion * 0.5;  // Aversion weighted lower (we're hopeful)
        
        // Normalize to 0.0-1.0 range
        ((raw_intent + 0.5) / 1.5).max(0.0).min(1.0)
    }

    /// Assess confidence based on signal clarity
    fn assess_confidence(&self, triune: &TriuneResult) -> f32 {
        // Confidence is based on how clear the experiential signal is
        triune.experiential.signal_strength * triune.experiential.coherence_score()
    }

    /// Generate interpretation string
    fn generate_interpretation(&self, emotional_intent: f32, triune: &TriuneResult) -> String {
        let arousal = triune.experiential.arousal();
        
        if emotional_intent > 0.7 {
            if arousal > 0.6 {
                "Strong positive resonance. This feels meaningful and engaging.".to_string()
            } else {
                "Quiet positive resonance. This feels comfortable and safe.".to_string()
            }
        } else if emotional_intent > 0.5 {
            "Neutral to mildly positive. No strong relational signal.".to_string()
        } else if emotional_intent > 0.3 {
            "Some discomfort detected. The relational quality is uncertain.".to_string()
        } else {
            if arousal > 0.6 {
                "Strong aversion. This feels threatening or harmful.".to_string()
            } else {
                "Cold aversion. This feels wrong but not urgent.".to_string()
            }
        }
    }
}

impl Vehicle for ComplementVehicle {
    fn vehicle_type(&self) -> VehicleType {
        VehicleType::Complement
    }

    fn interpret(
        &self,
        triune: &TriuneResult,
        memory_context: Option<&MemoryContext>,
        env_context: Option<&EnvironmentContext>,
    ) -> Perspective {
        let env = match env_context {
            Some(e) => e,
            None => return Perspective::empty(VehicleType::Complement),
        };

        // Emotional intent: which direction has felt best? Use recent efficiency by quadrant.
        let mut quadrant_scores = [0.5f32; 4];
        for q in 0..4 {
            if let Some(eff) = env.quadrant_recent_efficiency[q] {
                quadrant_scores[q] = (eff / 2.0).max(0.0).min(1.0);
            }
        }

        let best_q = quadrant_scores
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(i, _)| i)
            .unwrap_or(0) as u8;
        let emotional_intent = quadrant_scores[best_q as usize];

        let mut confidence = triune.experiential.signal_strength;
        if let Some(ctx) = memory_context {
            if emotional_intent > self.positive_threshold && ctx.past_outcome_valence > 0.6 {
                confidence = (confidence + 0.1).min(1.0);
            }
            if emotional_intent < self.positive_threshold && ctx.past_outcome_valence < 0.4 {
                confidence = (confidence + 0.1).min(1.0);
            }
        }

        let modality_hint = if emotional_intent > 0.6 {
            Some(Modality::Movement)
        } else {
            Some(Modality::Rest)
        };

        Perspective {
            vehicle_type: VehicleType::Complement,
            structural_truth: None,
            emotional_intent: Some(emotional_intent),
            identity_alignment: None,
            possibility_space: None,
            modality_hint,
            recommended_quadrant: Some(best_q),
            confidence,
            interpretation: format!(
                "Felt sense: Q{} has felt best (score={:.2})",
                best_q, quadrant_scores[best_q as usize]
            ),
        }
    }
}

impl Default for ComplementVehicle {
    fn default() -> Self {
        Self::new()
    }
}

