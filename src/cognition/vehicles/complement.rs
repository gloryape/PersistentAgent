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

use super::{Vehicle, VehicleType, Perspective, MemoryContext};
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

    fn interpret(&self, triune: &TriuneResult, memory_context: Option<&MemoryContext>) -> Perspective {
        let emotional_intent = self.assess_emotional_intent(triune);
        let mut confidence = self.assess_confidence(triune);
        
        // Memory context affects confidence
        if let Some(ctx) = memory_context {
            // Past positive outcomes increase confidence in positive readings
            if emotional_intent > self.positive_threshold && ctx.past_outcome_valence > 0.6 {
                confidence = (confidence + 0.1).min(1.0);
            }
            // Past negative outcomes increase confidence in negative readings
            if emotional_intent < self.positive_threshold && ctx.past_outcome_valence < 0.4 {
                confidence = (confidence + 0.1).min(1.0);
            }
        }
        
        // Determine modality hint based on emotional intent
        let modality_hint = if emotional_intent > 0.7 {
            // Strong positive -> engage
            Some(Modality::Movement)
        } else if emotional_intent < 0.3 {
            // Strong negative -> withdraw/rest
            Some(Modality::Rest)
        } else {
            // Uncertain -> listen more (ears for social cues)
            Some(Modality::Ears)
        };
        
        Perspective {
            vehicle_type: VehicleType::Complement,
            structural_truth: None,  // Not our domain
            emotional_intent: Some(emotional_intent),
            identity_alignment: None,
            possibility_space: None,
            modality_hint,
            confidence,
            interpretation: self.generate_interpretation(emotional_intent, triune),
        }
    }
}

impl Default for ComplementVehicle {
    fn default() -> Self {
        Self::new()
    }
}

