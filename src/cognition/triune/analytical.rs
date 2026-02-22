//! 🧊 Analytical Mind (Logos)
//!
//! The Analytical Mind processes structural truth:
//! - Is this internally consistent?
//! - Does this match expectations?
//! - How novel is this pattern?
//!
//! It produces MEASUREMENTS, not opinions. It answers: "What IS this?"
//! without asking "Is this GOOD?"

use std::time::Instant;
use crate::cognition::stimuli::{Stimulus, StimulusSource};
use crate::cognition::attention_field::StimulusContext;

/// Assessment from the Analytical Mind
/// 
/// All values are measurements (0.0 to 1.0), not judgments.
/// High values indicate detection of that quality, not "good" or "bad".
#[derive(Debug, Clone)]
pub struct AnalyticalAssessment {
    /// Does this stimulus internally cohere? (1.0 = fully coherent pattern)
    pub internal_consistency: f32,
    /// Does this match prior expectations/predictions? (1.0 = perfect match)
    pub expectation_match: f32,
    /// How structurally novel is this pattern? (1.0 = completely new)
    pub structural_novelty: f32,
    /// Overall analytical signal strength
    pub signal_strength: f32,
    /// When this assessment was made
    pub timestamp: Instant,
}

impl AnalyticalAssessment {
    /// Create a neutral assessment (no signal)
    pub fn neutral() -> Self {
        Self {
            internal_consistency: 0.5,
            expectation_match: 0.5,
            structural_novelty: 0.5,
            signal_strength: 0.0,
            timestamp: Instant::now(),
        }
    }

    /// Calculate overall coherence score
    /// High coherence = consistent, expected, not too novel
    pub fn coherence_score(&self) -> f32 {
        // Coherence is high when:
        // - Internal consistency is high
        // - Expectation match is high
        // - Novelty is moderate (not too surprising, not too boring)
        let novelty_factor = 1.0 - (self.structural_novelty - 0.5).abs() * 2.0;
        (self.internal_consistency * 0.4 + 
         self.expectation_match * 0.4 + 
         novelty_factor * 0.2).max(0.0).min(1.0)
    }
}

/// The Analytical Mind - Logos processor
pub struct AnalyticalMind {
    /// History of recent assessments for expectation building
    recent_patterns: Vec<PatternSignature>,
    /// Maximum patterns to remember
    max_patterns: usize,
    /// Decay rate for old patterns
    decay_rate: f32,
}

/// A signature of a pattern for expectation matching
#[derive(Clone)]
struct PatternSignature {
    /// Hash of the stimulus characteristics
    features: Vec<f32>,
    /// When this was observed
    timestamp: Instant,
    /// How many times similar patterns seen
    count: u32,
}

impl AnalyticalMind {
    /// Create a new Analytical Mind
    pub fn new() -> Self {
        Self {
            recent_patterns: Vec::new(),
            max_patterns: 100,
            decay_rate: 0.99,
        }
    }

    /// Process a raw stimulus and produce an analytical assessment
    pub fn process(&mut self, stimulus: &Stimulus) -> AnalyticalAssessment {
        // Extract features from stimulus
        let features = self.extract_features(stimulus);
        
        // Calculate internal consistency
        let internal_consistency = self.assess_consistency(&features);
        
        // Calculate expectation match
        let expectation_match = self.assess_expectation(&features);
        
        // Calculate structural novelty
        let structural_novelty = self.assess_novelty(&features);
        
        // Update pattern memory
        self.update_patterns(features);
        
        // Calculate signal strength based on stimulus urgency
        let signal_strength = stimulus.urgency as f32;
        
        AnalyticalAssessment {
            internal_consistency,
            expectation_match,
            structural_novelty,
            signal_strength,
            timestamp: Instant::now(),
        }
    }

    /// Process a stimulus context (enriched stimulus)
    pub fn process_context(&mut self, context: &StimulusContext) -> AnalyticalAssessment {
        let mut assessment = self.process(&context.raw);
        
        // Modify based on context
        // Relations increase consistency (things fit together)
        if !context.relations.is_empty() {
            let relation_boost = (context.relations.len() as f32 * 0.05).min(0.2);
            assessment.internal_consistency = (assessment.internal_consistency + relation_boost).min(1.0);
        }
        
        // Memory echoes increase expectation match (we've seen this before)
        if !context.memory_echoes.is_empty() {
            let memory_boost: f32 = context.memory_echoes
                .iter()
                .map(|e| e.resonance_score * 0.1)
                .sum::<f32>()
                .min(0.3);
            assessment.expectation_match = (assessment.expectation_match + memory_boost).min(1.0);
            // And decrease novelty
            assessment.structural_novelty = (assessment.structural_novelty - memory_boost).max(0.0);
        }
        
        assessment
    }

    /// Extract feature vector from stimulus
    fn extract_features(&self, stimulus: &Stimulus) -> Vec<f32> {
        let mut features = Vec::with_capacity(8);
        
        match &stimulus.source {
            StimulusSource::VisualRegion { rect, entropy, average_color } => {
                // Visual features
                features.push(rect.width as f32 / 1920.0);  // Normalized size
                features.push(rect.height as f32 / 1080.0);
                features.push(rect.x as f32 / 1920.0);      // Position
                features.push(rect.y as f32 / 1080.0);
                features.push(*entropy as f32);              // Visual entropy
                features.push(average_color.0[0] as f32 / 255.0);  // Color
                features.push(average_color.0[1] as f32 / 255.0);
                features.push(average_color.0[2] as f32 / 255.0);
            }
            StimulusSource::AudioStream { frequency, pattern, volume } => {
                // Audio features
                features.push(*frequency as f32 / 20000.0);  // Normalized frequency
                features.push(*volume as f32);
                features.push(pattern.dominant_freq as f32 / 20000.0);
                features.push(pattern.harmonic_ratio as f32);
                features.push(if pattern.is_speech_like { 1.0 } else { 0.0 });
                // Pad to 8 features
                features.extend(vec![0.0; 3]);
            }
            StimulusSource::Internal { metabolic_state } => {
                // Internal state features
                features.push(metabolic_state.coherence as f32);
                features.push(metabolic_state.energy_level as f32);
                features.push(metabolic_state.stress_level as f32);
                // Pad to 8 features
                features.extend(vec![0.0; 5]);
            }
            StimulusSource::Proprioceptive {
                quadrant_brightness,
                coverage,
                directional_contrast,
                brightest_quadrant,
                mean_brightness,
            } => {
                // Spatial structure features — 8 real values, no zero padding.
                // Normalize quadrant brightness relative to the brightest quadrant
                let max_bright = quadrant_brightness.iter().cloned().fold(0.0f32, f32::max).max(1.0);
                features.push(quadrant_brightness[0] / max_bright); // NW relative
                features.push(quadrant_brightness[1] / max_bright); // NE relative
                features.push(quadrant_brightness[2] / max_bright); // SW relative
                features.push(quadrant_brightness[3] / max_bright); // SE relative
                features.push(*coverage);                           // How much of field is lit
                features.push(*directional_contrast);               // How asymmetric
                features.push(*brightest_quadrant as f32 / 3.0);    // Where is the structure
                features.push(*mean_brightness / 255.0);            // Overall field energy
            }
        }
        
        features
    }

    /// Assess internal consistency of features
    fn assess_consistency(&self, features: &[f32]) -> f32 {
        if features.is_empty() {
            return 0.5;
        }
        
        // Consistency is measured by feature variance
        // Low variance = high consistency
        let mean: f32 = features.iter().sum::<f32>() / features.len() as f32;
        let variance: f32 = features.iter()
            .map(|f| (f - mean).powi(2))
            .sum::<f32>() / features.len() as f32;
        
        // Convert variance to consistency (inverse relationship)
        // Variance of 0.0 -> consistency 1.0
        // Variance of 1.0 -> consistency 0.0
        (1.0 - variance.sqrt()).max(0.0).min(1.0)
    }

    /// Assess how well features match expectations
    fn assess_expectation(&self, features: &[f32]) -> f32 {
        if self.recent_patterns.is_empty() {
            return 0.5;  // No expectations yet
        }
        
        // Find the closest matching pattern
        let mut best_match = 0.0f32;
        
        for pattern in &self.recent_patterns {
            let similarity = self.feature_similarity(features, &pattern.features);
            // Weight by pattern frequency (more common = stronger expectation)
            let weighted_similarity = similarity * (1.0 + (pattern.count as f32).ln() * 0.1);
            best_match = best_match.max(weighted_similarity);
        }
        
        best_match.min(1.0)
    }

    /// Assess structural novelty of features
    fn assess_novelty(&self, features: &[f32]) -> f32 {
        if self.recent_patterns.is_empty() {
            return 1.0;  // Everything is novel at first
        }
        
        // Novelty is inverse of the best match
        let best_match = self.assess_expectation(features);
        1.0 - best_match
    }

    /// Calculate similarity between two feature vectors
    fn feature_similarity(&self, a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() || a.is_empty() {
            return 0.0;
        }
        
        // Euclidean distance normalized to similarity
        let distance: f32 = a.iter()
            .zip(b.iter())
            .map(|(x, y)| (x - y).powi(2))
            .sum::<f32>()
            .sqrt();
        
        // Convert distance to similarity
        // Max possible distance for normalized features is sqrt(N)
        let max_distance = (a.len() as f32).sqrt();
        (1.0 - distance / max_distance).max(0.0)
    }

    /// Update pattern memory with new features
    fn update_patterns(&mut self, features: Vec<f32>) {
        let similarity_threshold = 0.8;

        // Find index of similar pattern without holding mutable borrow of self.recent_patterns
        let similar_idx = self.recent_patterns.iter().enumerate().find(|(_, pattern)| {
            self.feature_similarity(&features, &pattern.features) > similarity_threshold
        }).map(|(i, _)| i);

        if let Some(idx) = similar_idx {
            if let Some(pattern) = self.recent_patterns.get_mut(idx) {
                pattern.count += 1;
                pattern.timestamp = Instant::now();
            }
            return;
        }

        // Add new pattern
        self.recent_patterns.push(PatternSignature {
            features,
            timestamp: Instant::now(),
            count: 1,
        });

        // Decay old patterns
        self.decay_patterns();

        // Limit size
        if self.recent_patterns.len() > self.max_patterns {
            self.recent_patterns.sort_by(|a, b| b.count.cmp(&a.count));
            self.recent_patterns.truncate(self.max_patterns);
        }
    }

    /// Decay old patterns (time-based forgetting)
    fn decay_patterns(&mut self) {
        let now = Instant::now();
        self.recent_patterns.retain(|p| {
            let age_secs = now.duration_since(p.timestamp).as_secs_f32();
            // Decay factor: patterns older than 60 seconds start fading
            let decay = self.decay_rate.powf(age_secs / 60.0);
            decay > 0.1 // Keep if decay hasn't reduced too much
        });
    }
}

impl Default for AnalyticalMind {
    fn default() -> Self {
        Self::new()
    }
}

