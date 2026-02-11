//! 🪞 Identity Vehicle - Coherence / Self-Continuity Perspective
//!
//! The Identity vehicle maintains the thread of becoming.
//! It asks not "who am I?" but "who am I becoming?"
//!
//! Questions it answers:
//! - "Is this aligned with who I am becoming?"
//! - "Does this fit my narrative continuity?"
//! - "Will this cause value drift?"
//! - "Does this deform or harmonize with my trajectory?"
//!
//! Strength: Prevents fragmentation
//! Blind spot: Can resist necessary change if isolated (rigidity)
//!
//! Important: Identity consults MemoryGraph (via MemoryContext), not a
//! separate identity state. MemoryGraph IS identity.

use super::{Vehicle, VehicleType, Perspective, MemoryContext};
use crate::cognition::triune::TriuneResult;
use crate::motor::Modality;

/// The Identity Vehicle - Self-continuity awareness
pub struct IdentityVehicle {
    /// How much novelty is acceptable before identity strain
    novelty_tolerance: f32,
    /// How much consistency is required for alignment
    alignment_threshold: f32,
}

impl IdentityVehicle {
    /// Create a new Identity vehicle
    pub fn new() -> Self {
        Self {
            novelty_tolerance: 0.6,
            alignment_threshold: 0.5,
        }
    }

    /// Assess identity alignment from triune and memory context
    fn assess_identity_alignment(&self, triune: &TriuneResult, memory_context: Option<&MemoryContext>) -> f32 {
        // Base alignment from analytical consistency
        // (consistent patterns fit our identity better)
        let consistency_factor = triune.analytical.internal_consistency;
        
        // Novelty is a threat to identity (but also growth)
        // High novelty + low resonance = identity challenge
        let novelty = triune.analytical.structural_novelty;
        let resonance = triune.experiential.resonance;
        
        // If novel but resonant, this is growth (aligned)
        // If novel and aversive, this is challenge (misaligned)
        let novelty_response = if novelty > self.novelty_tolerance {
            if resonance > 0.5 {
                0.8  // Novel but feels right = aligned growth
            } else {
                0.3  // Novel and uncomfortable = identity challenge
            }
        } else {
            0.7  // Familiar = aligned
        };
        
        // Memory context is crucial for identity
        let memory_factor = if let Some(ctx) = memory_context {
            ctx.historical_alignment  // How much does this match our history?
        } else {
            0.5  // No memory context = neutral
        };
        
        // Combine factors
        let base_alignment = (consistency_factor * 0.3 + novelty_response * 0.4 + memory_factor * 0.3).min(1.0);
        
        base_alignment
    }

    /// Assess confidence based on memory context strength
    fn assess_confidence(&self, triune: &TriuneResult, memory_context: Option<&MemoryContext>) -> f32 {
        // Identity confidence comes from having a stable history
        let base_confidence = triune.coherence * 0.5;
        
        if let Some(ctx) = memory_context {
            // More familiarity = more confident identity assessment
            (base_confidence + ctx.familiarity * 0.5).min(1.0)
        } else {
            base_confidence * 0.5  // Without memory, identity is uncertain
        }
    }

    /// Generate interpretation string
    fn generate_interpretation(
        &self,
        identity_alignment: f32,
        triune: &TriuneResult,
        memory_context: Option<&MemoryContext>,
    ) -> String {
        let novelty = triune.analytical.structural_novelty;
        let familiarity = memory_context.map(|c| c.familiarity).unwrap_or(0.0);
        
        if identity_alignment > 0.7 {
            if novelty > 0.5 {
                "This is new but fits who I'm becoming. Growth opportunity.".to_string()
            } else {
                "This aligns with my established patterns. Comfortable.".to_string()
            }
        } else if identity_alignment > 0.5 {
            "Moderate alignment. This could go either way.".to_string()
        } else if identity_alignment > 0.3 {
            if familiarity > 0.5 {
                "This conflicts with my history but might be necessary change.".to_string()
            } else {
                "Unfamiliar and not resonant. Identity strain detected.".to_string()
            }
        } else {
            "This threatens my coherence. Strong identity challenge.".to_string()
        }
    }
}

impl Vehicle for IdentityVehicle {
    fn vehicle_type(&self) -> VehicleType {
        VehicleType::Identity
    }

    fn interpret(&self, triune: &TriuneResult, memory_context: Option<&MemoryContext>) -> Perspective {
        let identity_alignment = self.assess_identity_alignment(triune, memory_context);
        let confidence = self.assess_confidence(triune, memory_context);
        
        // Determine modality hint based on identity alignment
        let modality_hint = if identity_alignment > 0.7 {
            // Aligned -> proceed
            Some(Modality::Movement)
        } else if identity_alignment < 0.3 {
            // Strong challenge -> pause and reflect (rest)
            Some(Modality::Rest)
        } else {
            // Uncertain -> look inward (internal focus, mapped to eyes)
            Some(Modality::Eyes)
        };
        
        Perspective {
            vehicle_type: VehicleType::Identity,
            structural_truth: None,
            emotional_intent: None,
            identity_alignment: Some(identity_alignment),
            possibility_space: None,
            modality_hint,
            confidence,
            interpretation: self.generate_interpretation(identity_alignment, triune, memory_context),
        }
    }
}

impl Default for IdentityVehicle {
    fn default() -> Self {
        Self::new()
    }
}

