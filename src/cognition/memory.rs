//! 🧠 Memory Bank - Learning and Correlation
//!
//! Stores learned correlations between visual shapes and audio sounds.
//! The organism learns through literal experience, not symbolic definitions.
//!
//! Also provides MemoryReference for the AttentionField to query associations.

use uuid::Uuid;
use std::time::Instant;

// Forward declare MotorImpulse type for MemoryReference
// (actual type is in motor module)
use crate::motor::MotorImpulse;

/// A reference to a memory that resonates with a stimulus
/// Used by AttentionField to enrich stimuli with past experience
#[derive(Debug, Clone)]
pub struct MemoryReference {
    /// The memory crystal ID
    pub id: Uuid,
    /// How strongly this memory resonates (emotional weight)
    pub resonance_score: f32,
    /// Pattern match quality (specific = 1.0, category = 0.5)
    pub similarity: f32,
    /// What action was taken when this memory was formed (primes motor cortex)
    pub outcome_vector: Option<MotorImpulse>,
}

/// A MemoryCrystal stores a learned correlation between visual and audio patterns.
#[derive(Debug, Clone)]
pub struct MemoryCrystal {
    pub id: Uuid,
    pub visual_signature: VisualSignature,
    pub audio_signature: AudioSignature,
    pub correlation_score: f64,  // 0.0 to 1.0 (how strong is the association?)
    pub confidence: f64,         // 0.0 to 1.0 (how certain are we?)
    pub first_observed: Instant,
    pub observation_count: u32,
}

/// Visual signature describing a shape/pattern.
#[derive(Debug, Clone)]
pub struct VisualSignature {
    pub shape_features: Vec<f64>, // Feature vector describing the shape
    pub color_features: Vec<f64>, // Color histogram (normalized)
    pub spatial_features: Vec<f64>, // Position, size, aspect ratio, etc.
}

/// Audio signature describing a sound pattern.
#[derive(Debug, Clone)]
pub struct AudioSignature {
    pub frequency_profile: Vec<f64>, // Frequency spectrum (normalized)
    pub temporal_pattern: Vec<f64>,  // Time-domain features
    pub dominant_freq: f64,
}

/// Memory bank that stores learned correlations.
pub struct MemoryBank {
    crystals: Vec<MemoryCrystal>,
    max_crystals: usize,
}

impl MemoryBank {
    /// Create a new MemoryBank.
    pub fn new() -> Self {
        Self {
            crystals: Vec::new(),
            max_crystals: 1000,
        }
    }

    /// Record a visual-audio correlation.
    ///
    /// If a similar memory exists, it is strengthened. Otherwise, a new
    /// memory crystal is created. This is how the organism learns through
    /// repeated observation.
    pub fn record_correlation(
        &mut self,
        visual: &VisualSignature,
        audio: &AudioSignature,
    ) {
        // Check if similar memory exists
        if let Some(existing) = self.find_similar(visual, audio) {
            // Update existing memory (strengthen correlation)
            existing.observation_count += 1;
            // Correlation score increases with each observation
            existing.correlation_score = (existing.correlation_score * 0.9 + 0.1).min(1.0);
            // Confidence also increases
            existing.confidence = (existing.confidence * 0.95 + 0.05).min(1.0);
        } else {
            // Create new memory crystal
            let crystal = MemoryCrystal {
                id: uuid::Uuid::new_v4(),
                visual_signature: visual.clone(),
                audio_signature: audio.clone(),
                correlation_score: 0.1, // Start weak
                confidence: 0.1,
                first_observed: Instant::now(),
                observation_count: 1,
            };
            
            self.crystals.push(crystal);
            
            // Limit memory size
            if self.crystals.len() > self.max_crystals {
                // Remove weakest memories (lowest correlation * confidence)
                self.crystals.sort_by(|a, b| {
                    let score_a = a.correlation_score * a.confidence;
                    let score_b = b.correlation_score * b.confidence;
                    score_a.partial_cmp(&score_b).unwrap_or(std::cmp::Ordering::Equal)
                });
                // Keep top memories
                self.crystals.drain(0..self.crystals.len() - self.max_crystals);
            }
        }
    }

    /// Find similar memory (for updating existing).
    ///
    /// Uses cosine similarity to find memories with similar visual and audio signatures.
    fn find_similar(
        &mut self,
        visual: &VisualSignature,
        audio: &AudioSignature,
    ) -> Option<&mut MemoryCrystal> {
        let idx = self.crystals.iter().enumerate().find(|(_, crystal)| {
            let visual_sim = self.visual_similarity(&crystal.visual_signature, visual);
            let audio_sim = self.audio_similarity(&crystal.audio_signature, audio);
            visual_sim > 0.7 && audio_sim > 0.7
        })?.0;
        self.crystals.get_mut(idx)
    }

    /// Calculate visual similarity using cosine similarity.
    fn visual_similarity(&self, a: &VisualSignature, b: &VisualSignature) -> f64 {
        // Combine all feature vectors
        let features_a: Vec<f64> = a.shape_features.iter()
            .chain(a.color_features.iter())
            .chain(a.spatial_features.iter())
            .cloned()
            .collect();
        
        let features_b: Vec<f64> = b.shape_features.iter()
            .chain(b.color_features.iter())
            .chain(b.spatial_features.iter())
            .cloned()
            .collect();
        
        self.cosine_similarity(&features_a, &features_b)
    }

    /// Calculate audio similarity using cosine similarity.
    fn audio_similarity(&self, a: &AudioSignature, b: &AudioSignature) -> f64 {
        // Combine frequency profile and temporal pattern
        let features_a: Vec<f64> = a.frequency_profile.iter()
            .chain(a.temporal_pattern.iter())
            .cloned()
            .collect();
        
        let features_b: Vec<f64> = b.frequency_profile.iter()
            .chain(b.temporal_pattern.iter())
            .cloned()
            .collect();
        
        // Also consider dominant frequency similarity
        let freq_sim = 1.0 - ((a.dominant_freq - b.dominant_freq).abs() / 1000.0).min(1.0);
        
        // Combine spectrum similarity with frequency similarity
        (self.cosine_similarity(&features_a, &features_b) * 0.7 + freq_sim * 0.3).min(1.0)
    }

    /// Calculate cosine similarity between two feature vectors.
    fn cosine_similarity(&self, a: &[f64], b: &[f64]) -> f64 {
        if a.len() != b.len() || a.is_empty() {
            return 0.0;
        }

        let dot_product: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
        let norm_b: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }

        (dot_product / (norm_a * norm_b)).max(0.0).min(1.0)
    }

    /// Get all memory crystals.
    pub fn get_crystals(&self) -> &[MemoryCrystal] {
        &self.crystals
    }

    /// Get memory crystals with correlation score above threshold.
    pub fn get_strong_memories(&self, threshold: f64) -> Vec<&MemoryCrystal> {
        self.crystals.iter()
            .filter(|c| c.correlation_score >= threshold)
            .collect()
    }

    /// Get the number of stored memories.
    pub fn len(&self) -> usize {
        self.crystals.len()
    }
}

impl Default for MemoryBank {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryBank {
    /// Find memories that associate with a given visual or audio signature.
    /// Returns MemoryReferences for the AttentionField to use.
    ///
    /// Matching rules:
    /// - Exact match (similarity > 0.9) → resonance_score = crystal.correlation_score
    /// - Category match (similarity > 0.5) → resonance_score = crystal.correlation_score * 0.5
    pub fn find_associations_visual(&self, visual: &VisualSignature) -> Vec<MemoryReference> {
        let mut refs = Vec::new();
        
        for crystal in &self.crystals {
            let similarity = self.visual_similarity(&crystal.visual_signature, visual) as f32;
            
            if similarity > 0.5 {
                let resonance = if similarity > 0.9 {
                    crystal.correlation_score as f32
                } else {
                    crystal.correlation_score as f32 * 0.5
                };
                
                refs.push(MemoryReference {
                    id: crystal.id,
                    resonance_score: resonance,
                    similarity,
                    outcome_vector: None, // Will be filled by higher-level memory system
                });
            }
        }
        
        refs
    }

    /// Find memories that associate with a given audio signature.
    pub fn find_associations_audio(&self, audio: &AudioSignature) -> Vec<MemoryReference> {
        let mut refs = Vec::new();
        
        for crystal in &self.crystals {
            let similarity = self.audio_similarity(&crystal.audio_signature, audio) as f32;
            
            if similarity > 0.5 {
                let resonance = if similarity > 0.9 {
                    crystal.correlation_score as f32
                } else {
                    crystal.correlation_score as f32 * 0.5
                };
                
                refs.push(MemoryReference {
                    id: crystal.id,
                    resonance_score: resonance,
                    similarity,
                    outcome_vector: None,
                });
            }
        }
        
        refs
    }
}

