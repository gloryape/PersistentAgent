//! 🎯 Stimuli Queue - The Menu of Reality
//!
//! Converts raw sensory data into "Objects of Attention" that the Observer
//! can choose to engage with. Stimuli are prioritized by salience (urgency + novelty).

use uuid::Uuid;
use std::time::Instant;
use image::Rgb;

/// Source of a stimulus (what generated it)
#[derive(Debug, Clone)]
pub enum StimulusSource {
    VisualRegion {
        rect: Rect,
        entropy: f64,
        average_color: Rgb<u8>,
    },
    AudioStream {
        frequency: f64,
        pattern: AudioPattern,
        volume: f64,
    },
    Internal {
        metabolic_state: MetabolicState,
    },
}

/// Rectangular region (for visual stimuli)
#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl Rect {
    pub fn center(&self) -> (u32, u32) {
        (self.x + self.width / 2, self.y + self.height / 2)
    }

    pub fn area(&self) -> u32 {
        self.width * self.height
    }
}

/// Audio pattern descriptor
#[derive(Debug, Clone)]
pub struct AudioPattern {
    pub dominant_freq: f64,
    pub harmonic_ratio: f64,
    pub is_speech_like: bool,
}

/// Metabolic state descriptor
#[derive(Debug, Clone)]
pub struct MetabolicState {
    pub coherence: f64,
    pub energy_level: f64,
    pub stress_level: f64,
}

/// A stimulus represents an "Object of Attention"
#[derive(Debug, Clone)]
pub struct Stimulus {
    pub id: Uuid,
    pub source: StimulusSource,
    pub urgency: f64,      // 0.0 to 1.0 (How loud/bright is it?)
    pub novelty: f64,      // 0.0 to 1.0 (Have I seen this before?)
    pub salience: f64,     // Combined urgency + novelty score
    pub timestamp: Instant,
    pub attention_count: u32, // How many times has this been attended to?
}

use std::collections::BinaryHeap;
use std::cmp::Ordering;

/// Priority queue for stimuli, sorted by salience
pub struct StimuliQueue {
    queue: BinaryHeap<Stimulus>,
    max_size: usize,
    stimulus_history: Vec<Uuid>, // Track seen stimuli for novelty calculation
}

impl StimuliQueue {
    /// Create a new StimuliQueue with a maximum size
    pub fn new(max_size: usize) -> Self {
        Self {
            queue: BinaryHeap::new(),
            max_size,
            stimulus_history: Vec::new(),
        }
    }

    /// Add a stimulus to the queue
    pub fn add(&mut self, mut stimulus: Stimulus) {
        // Calculate novelty based on history
        stimulus.novelty = self.calculate_novelty(&stimulus);
        
        // Calculate salience (combined urgency + novelty)
        // Urgency weighted 60%, novelty weighted 40%
        stimulus.salience = (stimulus.urgency * 0.6 + stimulus.novelty * 0.4).min(1.0);
        
        // Add to queue (priority by salience)
        self.queue.push(stimulus);
        
        // Limit queue size (keep most salient)
        if self.queue.len() > self.max_size {
            let mut temp: Vec<_> = self.queue.drain().collect();
            temp.sort_by(|a, b| b.salience.partial_cmp(&a.salience).unwrap_or(Ordering::Equal));
            self.queue = temp.into_iter().take(self.max_size).collect();
        }
    }

    /// Add multiple stimuli at once
    pub fn add_batch(&mut self, stimuli: Vec<Stimulus>) {
        for stimulus in stimuli {
            self.add(stimulus);
        }
    }

    /// Peek at the most salient stimulus without removing it
    pub fn peek(&self) -> Option<&Stimulus> {
        self.queue.peek()
    }

    /// Remove and return the most salient stimulus
    pub fn pop(&mut self) -> Option<Stimulus> {
        self.queue.pop()
    }

    /// Calculate novelty: 1.0 = never seen, 0.0 = seen many times
    fn calculate_novelty(&self, stimulus: &Stimulus) -> f64 {
        let seen_count = self.stimulus_history.iter()
            .filter(|&id| *id == stimulus.id)
            .count();
        
        // Novelty decreases with each sighting
        // Formula: 1 / (1 + count) gives diminishing returns
        (1.0 / (1.0 + seen_count as f64)).min(1.0)
    }

    /// Mark a stimulus as attended (updates history for novelty calculation)
    pub fn mark_attended(&mut self, stimulus_id: Uuid) {
        self.stimulus_history.push(stimulus_id);
        // Keep history limited to prevent unbounded growth
        if self.stimulus_history.len() > 1000 {
            self.stimulus_history.drain(0..500);
        }
    }

    /// Get the current queue size
    pub fn len(&self) -> usize {
        self.queue.len()
    }

    /// Check if queue is empty
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
}

// Implement Ord for Stimulus (priority by salience, higher = more important)
impl Ord for Stimulus {
    fn cmp(&self, other: &Self) -> Ordering {
        self.salience.partial_cmp(&other.salience)
            .unwrap_or(Ordering::Equal)
    }
}

impl PartialOrd for Stimulus {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Stimulus {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Stimulus {}

