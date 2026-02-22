//! ❤️ Experiential Heart (Resonance)
//!
//! The Experiential Heart processes felt sense:
//! - Does this resonate (draw toward)?
//! - Does this avert (push away)?
//! - How salient is this to my being?
//!
//! It produces SIGNALS, not emotions. It answers: "How does this FEEL?"
//! without asking "What does this MEAN?"

use std::time::Instant;
use crate::cognition::stimuli::{Stimulus, StimulusSource};
use crate::cognition::attention_field::StimulusContext;

/// Assessment from the Experiential Heart
///
/// All values are signal strengths (0.0 to 1.0), not emotions.
/// These are physiological-analog responses, not semantic judgments.
#[derive(Debug, Clone)]
pub struct ExperientialAssessment {
    /// Resonance: Draw-toward signal (1.0 = strong pull)
    pub resonance: f32,
    /// Aversion: Push-away signal (1.0 = strong repulsion)
    pub aversion: f32,
    /// Salience: How much this demands attention (1.0 = can't ignore)
    pub salience: f32,
    /// Overall experiential signal strength
    pub signal_strength: f32,
    /// When this assessment was made
    pub timestamp: Instant,
}

impl ExperientialAssessment {
    /// Create a neutral assessment (no signal)
    pub fn neutral() -> Self {
        Self {
            resonance: 0.5,
            aversion: 0.5,
            salience: 0.0,
            signal_strength: 0.0,
            timestamp: Instant::now(),
        }
    }

    /// Calculate net valence (positive = resonance, negative = aversion)
    pub fn valence(&self) -> f32 {
        self.resonance - self.aversion
    }

    /// Calculate overall arousal (how activated is the system?)
    pub fn arousal(&self) -> f32 {
        // Arousal is the sum of resonance and aversion
        // Both attraction and repulsion activate the system
        (self.resonance + self.aversion).min(1.0)
    }

    /// Calculate experiential coherence
    /// High coherence = clear signal (either resonance OR aversion, not both)
    pub fn coherence_score(&self) -> f32 {
        // Coherence is high when:
        // - We have a clear valence (not ambivalent)
        // - Signal strength is meaningful
        let valence_clarity = (self.resonance - self.aversion).abs();
        (valence_clarity * self.signal_strength).max(0.0).min(1.0)
    }
}

/// The Experiential Heart - Resonance processor
pub struct ExperientialHeart {
    /// Baseline comfort level (what we're used to)
    baseline: BaselineState,
    /// Recent emotional history for adaptation
    recent_experiences: Vec<ExperienceRecord>,
    /// Maximum experiences to remember
    max_experiences: usize,
}

/// Baseline state for comparison
struct BaselineState {
    /// Typical visual entropy (what we're comfortable with)
    typical_visual_entropy: f32,
    /// Typical audio volume
    typical_audio_volume: f32,
    /// Typical change rate
    typical_change_rate: f32,
    /// Adaptation rate
    adaptation_rate: f32,
}

/// Record of a past experience
struct ExperienceRecord {
    /// Resonance level
    resonance: f32,
    /// Aversion level
    aversion: f32,
    /// When this was recorded
    timestamp: Instant,
}

impl ExperientialHeart {
    /// Create a new Experiential Heart
    pub fn new() -> Self {
        Self {
            baseline: BaselineState {
                typical_visual_entropy: 0.3,   // Natural scenes
                typical_audio_volume: 0.3,     // Moderate volume
                typical_change_rate: 0.2,      // Moderate change
                adaptation_rate: 0.01,         // Slow adaptation
            },
            recent_experiences: Vec::new(),
            max_experiences: 50,
        }
    }

    /// Process a raw stimulus and produce an experiential assessment
    pub fn process(&mut self, stimulus: &Stimulus) -> ExperientialAssessment {
        let (resonance, aversion, salience) = match &stimulus.source {
            StimulusSource::VisualRegion { entropy, rect, average_color } => {
                self.assess_visual(*entropy as f32, rect.area() as f32, average_color)
            }
            StimulusSource::AudioStream { volume, frequency, pattern } => {
                self.assess_audio(*volume as f32, *frequency as f32, pattern.harmonic_ratio as f32)
            }
            StimulusSource::Internal { metabolic_state } => {
                self.assess_internal(metabolic_state.coherence as f32, metabolic_state.stress_level as f32)
            }
            StimulusSource::Proprioceptive {
                directional_contrast,
                coverage,
                mean_brightness,
                ..
            } => {
                self.assess_proprioceptive(*directional_contrast, *coverage, *mean_brightness)
            }
        };
        
        // Calculate signal strength based on urgency and salience
        let signal_strength = (stimulus.urgency as f32 * 0.5 + salience * 0.5).min(1.0);
        
        // Update baseline (slow adaptation)
        self.adapt_baseline(resonance, aversion);
        
        // Record experience
        self.record_experience(resonance, aversion);
        
        ExperientialAssessment {
            resonance,
            aversion,
            salience,
            signal_strength,
            timestamp: Instant::now(),
        }
    }

    /// Process a stimulus context (enriched stimulus)
    pub fn process_context(&mut self, context: &StimulusContext) -> ExperientialAssessment {
        let mut assessment = self.process(&context.raw);
        
        // Memory echoes affect resonance (familiar = comfortable or aversive based on past)
        for echo in &context.memory_echoes {
            if echo.resonance_score > 0.6 {
                // Positive memory echo -> increase resonance
                assessment.resonance = (assessment.resonance + echo.resonance_score * 0.1).min(1.0);
            } else if echo.resonance_score < 0.4 {
                // Negative memory echo -> increase aversion
                assessment.aversion = (assessment.aversion + (1.0 - echo.resonance_score) * 0.1).min(1.0);
            }
        }
        
        // Relations affect salience (connected things are more important)
        if !context.relations.is_empty() {
            let relation_boost = (context.relations.len() as f32 * 0.05).min(0.2);
            assessment.salience = (assessment.salience + relation_boost).min(1.0);
        }
        
        assessment
    }

    /// Assess visual stimulus
    fn assess_visual(&self, entropy: f32, area: f32, color: &image::Rgb<u8>) -> (f32, f32, f32) {
        // Visual comfort zone: moderate entropy (not boring, not chaotic)
        let entropy_deviation = (entropy - self.baseline.typical_visual_entropy).abs();
        
        // Resonance: We like visual harmony, moderate complexity
        let mut resonance = (1.0 - entropy_deviation).max(0.0);
        
        // Color affects mood
        // Warm colors (red, orange) -> slightly more activating
        // Cool colors (blue, green) -> slightly more calming
        let warmth = (color.0[0] as f32 - color.0[2] as f32) / 255.0;
        resonance = (resonance + warmth * 0.1).max(0.0).min(1.0);
        
        // Aversion: Extreme entropy (chaos) or too much area (overwhelming)
        let mut aversion = 0.0;
        if entropy > 0.8 {
            aversion = (entropy - 0.8) * 2.5;  // High entropy -> aversion
        }
        if area > 500000.0 {  // Large area
            aversion = (aversion + (area - 500000.0) / 1000000.0).min(1.0);
        }
        
        // Salience: Based on area and entropy deviation
        let normalized_area = (area / 1000000.0).min(1.0);
        let salience = (entropy_deviation * 0.5 + normalized_area * 0.5).min(1.0);
        
        (resonance, aversion, salience)
    }

    /// Assess audio stimulus
    fn assess_audio(&self, volume: f32, frequency: f32, harmonic_ratio: f32) -> (f32, f32, f32) {
        // Volume comfort zone
        let volume_deviation = (volume - self.baseline.typical_audio_volume).abs();
        
        // Resonance: Harmonic sounds (musical), moderate volume
        let mut resonance = harmonic_ratio * 0.6 + (1.0 - volume_deviation) * 0.4;
        
        // Frequency affects comfort
        // Mid-range frequencies (speech range ~85-255 Hz fundamental) are comfortable
        // Very high or very low can be uncomfortable
        let freq_normalized = frequency / 20000.0;
        if freq_normalized > 0.3 {  // High pitched
            resonance = (resonance - (freq_normalized - 0.3) * 0.5).max(0.0);
        }
        
        // Aversion: Sudden loud sounds, dissonance
        let mut aversion = 0.0;
        if volume > 0.8 {
            aversion = (volume - 0.8) * 2.5;  // Loud -> aversion
        }
        // Low harmonic ratio (dissonant) -> aversion
        if harmonic_ratio < 0.3 {
            aversion = (aversion + (0.3 - harmonic_ratio) * 0.5).min(1.0);
        }
        
        // Salience: Loud sounds and unusual frequencies are salient
        let salience = (volume * 0.6 + volume_deviation * 0.4).min(1.0);
        
        (resonance, aversion, salience)
    }

    /// Assess internal metabolic stimulus
    fn assess_internal(&self, coherence: f32, stress: f32) -> (f32, f32, f32) {
        // Internal coherence feels good
        let resonance = coherence;
        
        // Stress feels bad
        let aversion = stress;
        
        // Internal states are always salient
        let salience = (coherence + stress) * 0.5;
        
        (resonance, aversion, salience)
    }

    /// Assess proprioceptive spatial awareness
    /// This is a felt sense, not a tropism. The Heart reports "how engaging is this
    /// spatial structure" — whether the organism acts on it depends entirely on whether
    /// the Triune achieves coherence and the Observer reaches readiness threshold.
    fn assess_proprioceptive(&self, directional_contrast: f32, coverage: f32, mean_brightness: f32) -> (f32, f32, f32) {
        // Directional structure is engaging — the organism senses a gradient.
        // Higher contrast = stronger felt pull toward exploration.
        let resonance = directional_contrast.min(1.0);

        // Low coverage in void isn't aversive — it's just sparse.
        // Very high coverage might indicate the organism is boxed in (slight aversion).
        let aversion = if coverage > 0.1 { (coverage - 0.1) * 0.5 } else { 0.0 };

        // Salience: is there anything to notice at all?
        // Any directional structure is salient. Brightness indicates energy presence.
        let salience = (directional_contrast + coverage * 2.0 + mean_brightness / 255.0).min(1.0);

        (resonance, aversion, salience)
    }

    /// Slowly adapt baseline to current experience
    fn adapt_baseline(&mut self, resonance: f32, aversion: f32) {
        // Only adapt when not in extreme states
        if resonance < 0.9 && aversion < 0.9 {
            // Slight adaptation toward current experience
            // (We get used to things over time)
        }
    }

    /// Record experience for history
    fn record_experience(&mut self, resonance: f32, aversion: f32) {
        self.recent_experiences.push(ExperienceRecord {
            resonance,
            aversion,
            timestamp: Instant::now(),
        });
        
        // Limit history
        if self.recent_experiences.len() > self.max_experiences {
            self.recent_experiences.remove(0);
        }
    }

    /// Get the average recent experience (for trend detection)
    pub fn recent_trend(&self) -> (f32, f32) {
        if self.recent_experiences.is_empty() {
            return (0.5, 0.5);
        }
        
        let sum_resonance: f32 = self.recent_experiences.iter().map(|e| e.resonance).sum();
        let sum_aversion: f32 = self.recent_experiences.iter().map(|e| e.aversion).sum();
        let count = self.recent_experiences.len() as f32;
        
        (sum_resonance / count, sum_aversion / count)
    }
}

impl Default for ExperientialHeart {
    fn default() -> Self {
        Self::new()
    }
}

