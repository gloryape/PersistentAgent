//! 🔺 Triune Processor
//!
//! Coordinates the Analytical Mind and Experiential Heart to produce
//! a unified TriuneResult. This is NOT a decision-maker - it produces
//! measurements that the Observer-Witness uses for classification.
//!
//! Key insight: The Triune does not resolve dissonance. It DETECTS it.
//! Resolution (or acceptance of mystery) is the Observer's domain.

use std::time::Instant;
use super::analytical::{AnalyticalMind, AnalyticalAssessment};
use super::experiential::{ExperientialHeart, ExperientialAssessment};
use crate::cognition::stimuli::Stimulus;
use crate::cognition::attention_field::StimulusContext;

/// Combined result from Triune processing
#[derive(Debug, Clone)]
pub struct TriuneResult {
    /// Assessment from the Analytical Mind
    pub analytical: AnalyticalAssessment,
    /// Assessment from the Experiential Heart
    pub experiential: ExperientialAssessment,
    /// When this result was produced
    pub timestamp: Instant,
    /// Calculated dissonance level (0.0 = harmony, 1.0 = maximum conflict)
    pub dissonance: f32,
    /// Overall Triune coherence (how aligned are Mind and Heart?)
    pub coherence: f32,
}

impl TriuneResult {
    /// Create from assessments
    pub fn from_assessments(analytical: AnalyticalAssessment, experiential: ExperientialAssessment) -> Self {
        let dissonance = Self::calculate_dissonance(&analytical, &experiential);
        let coherence = Self::calculate_coherence(&analytical, &experiential);
        
        Self {
            analytical,
            experiential,
            timestamp: Instant::now(),
            dissonance,
            coherence,
        }
    }

    /// Calculate dissonance between Mind and Heart
    ///
    /// Dissonance occurs when:
    /// - Mind says "consistent/expected" but Heart says "aversive"
    /// - Mind says "novel/inconsistent" but Heart says "resonant"
    ///
    /// This is the "spark" that drives curiosity and growth.
    fn calculate_dissonance(analytical: &AnalyticalAssessment, experiential: &ExperientialAssessment) -> f32 {
        // Mind's "truth" signal: high consistency + high expectation = "true"
        let mind_truth = analytical.coherence_score();
        
        // Heart's "good" signal: resonance - aversion
        let heart_valence = experiential.valence();  // -1.0 to 1.0, normalized to 0.0-1.0
        let heart_good = (heart_valence + 1.0) / 2.0;
        
        // Dissonance is the gap between "true" and "good"
        // Mind says "this is true" (high) but Heart says "this feels bad" (low) = dissonance
        // Mind says "this is false" (low) but Heart says "this feels good" (high) = dissonance
        let gap = (mind_truth - heart_good).abs();
        
        // Weight by signal strengths (weak signals shouldn't create strong dissonance)
        let weight = (analytical.signal_strength + experiential.signal_strength) / 2.0;
        
        (gap * weight).min(1.0)
    }

    /// Calculate overall coherence of the Triune result
    fn calculate_coherence(analytical: &AnalyticalAssessment, experiential: &ExperientialAssessment) -> f32 {
        // Coherence is high when:
        // - Both Mind and Heart have clear signals
        // - The signals align (low dissonance)
        // - Both assessments are internally coherent
        
        let analytical_coherence = analytical.coherence_score();
        let experiential_coherence = experiential.coherence_score();
        
        // Signal clarity
        let signal_clarity = (analytical.signal_strength + experiential.signal_strength) / 2.0;
        
        // Calculate dissonance for this
        let dissonance = Self::calculate_dissonance(analytical, experiential);
        
        // Coherence = average of both coherences * signal clarity * (1 - dissonance)
        let base_coherence = (analytical_coherence + experiential_coherence) / 2.0;
        (base_coherence * signal_clarity * (1.0 - dissonance * 0.5)).max(0.0).min(1.0)
    }

    /// Check if this result represents significant dissonance
    /// Threshold: 0.4 (40% dissonance)
    pub fn has_dissonance(&self) -> bool {
        self.dissonance > 0.4
    }

    /// Check if Mind says "true" but Heart says "bad"
    /// This is the classic "uncomfortable truth" pattern
    pub fn is_uncomfortable_truth(&self) -> bool {
        let mind_truth = self.analytical.coherence_score() > 0.7;
        let heart_bad = self.experiential.valence() < -0.2;
        mind_truth && heart_bad
    }

    /// Check if Mind says "false/novel" but Heart says "good"
    /// This is the "pleasant illusion" or "intuitive leap" pattern
    pub fn is_pleasant_novelty(&self) -> bool {
        let mind_novel = self.analytical.structural_novelty > 0.7;
        let heart_good = self.experiential.valence() > 0.2;
        mind_novel && heart_good
    }
}

/// The Triune Processor - Coordinates Mind and Heart
pub struct TriuneProcessor {
    /// The Analytical Mind
    pub analytical: AnalyticalMind,
    /// The Experiential Heart
    pub experiential: ExperientialHeart,
}

impl TriuneProcessor {
    /// Create a new Triune Processor
    pub fn new() -> Self {
        Self {
            analytical: AnalyticalMind::new(),
            experiential: ExperientialHeart::new(),
        }
    }

    /// Process a raw stimulus through both aspects
    pub fn process(&mut self, stimulus: &Stimulus) -> TriuneResult {
        let analytical = self.analytical.process(stimulus);
        let experiential = self.experiential.process(stimulus);
        
        TriuneResult::from_assessments(analytical, experiential)
    }

    /// Process a stimulus context (enriched stimulus)
    pub fn process_context(&mut self, context: &StimulusContext) -> TriuneResult {
        let analytical = self.analytical.process_context(context);
        let experiential = self.experiential.process_context(context);
        
        TriuneResult::from_assessments(analytical, experiential)
    }

    /// Get the recent trend from the Experiential Heart
    pub fn experiential_trend(&self) -> (f32, f32) {
        self.experiential.recent_trend()
    }
}

impl Default for TriuneProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_neutral_triune_result() {
        let analytical = AnalyticalAssessment::neutral();
        let experiential = ExperientialAssessment::neutral();
        let result = TriuneResult::from_assessments(analytical, experiential);
        
        // Neutral assessments should have low dissonance
        assert!(result.dissonance < 0.5);
    }

    #[test]
    fn test_dissonance_detection() {
        // Create high analytical coherence but negative experiential
        let analytical = AnalyticalAssessment {
            internal_consistency: 0.9,
            expectation_match: 0.9,
            structural_novelty: 0.2,
            signal_strength: 0.8,
            timestamp: Instant::now(),
        };
        
        let experiential = ExperientialAssessment {
            resonance: 0.2,
            aversion: 0.8,
            salience: 0.7,
            signal_strength: 0.8,
            timestamp: Instant::now(),
        };
        
        let result = TriuneResult::from_assessments(analytical, experiential);
        
        // This should have dissonance (true but bad)
        assert!(result.has_dissonance());
        assert!(result.is_uncomfortable_truth());
    }
}

