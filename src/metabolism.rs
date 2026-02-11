//! 🫀 Metabolism - The Governor
//!
//! The Metabolism is the heartbeat of the organism. It measures the actual timing
//! of the 90Hz loop and calculates coherence based on jitter from the target interval.
//!
//! Coherence starts at 0.0 (Tabula Rasa) and must be earned through stable timing.

use std::time::{Duration, Instant};

/// The Metabolism struct manages the organism's heartbeat and coherence measurement.
pub struct Metabolism {
    coherence: f64,
    target_interval_ns: u64,
    interval_history: Vec<u64>,
    max_history: usize,
    birth_time: Instant,
    last_tick: Option<Instant>,
}

impl Metabolism {
    /// Create a new Metabolism instance.
    ///
    /// Starts with coherence = 0.0 (Tabula Rasa) and target interval of 11.11ms (90Hz).
    pub fn new() -> Self {
        // 90Hz = 11.11ms = 11,111,111 nanoseconds
        let target_interval_ns = 11_111_111;
        
        Self {
            coherence: 0.0, // Tabula Rasa - starts at zero
            target_interval_ns,
            interval_history: Vec::with_capacity(20),
            max_history: 20,
            birth_time: Instant::now(),
            last_tick: None,
        }
    }

    /// Perform one heartbeat tick.
    ///
    /// Measures the actual loop duration, calculates jitter, and updates coherence.
    /// Returns the current coherence value.
    pub fn tick(&mut self) -> f64 {
        let now = Instant::now();
        
        // Calculate actual interval since last tick (or since birth if first tick)
        let actual_interval = if let Some(last) = self.last_tick {
            now.duration_since(last)
        } else {
            // First tick - use time since birth
            now.duration_since(self.birth_time)
        };
        
        let actual_interval_ns = actual_interval.as_nanos() as u64;
        
        // Store in rolling history window
        self.interval_history.push(actual_interval_ns);
        if self.interval_history.len() > self.max_history {
            self.interval_history.remove(0);
        }
        
        // Calculate coherence based on jitter
        let actual_interval_ms = actual_interval.as_secs_f64() * 1000.0;
        let target_interval_ms = self.target_interval_ns as f64 / 1_000_000.0; // 11.11ms
        let jitter_ms = (actual_interval_ms - target_interval_ms).abs();
        
        // Coherence = 1.0 - (jitter / target), clamped to [0.0, 1.0]
        // Higher jitter = lower coherence
        self.coherence = (1.0 - (jitter_ms / target_interval_ms))
            .max(0.0)
            .min(1.0);
        
        self.last_tick = Some(now);
        
        self.coherence
    }

    /// Get the current coherence value.
    pub fn get_coherence(&self) -> f64 {
        self.coherence
    }

    /// Set coherence (used when restoring from a saved simulation).
    pub fn set_coherence(&mut self, c: f64) {
        self.coherence = c.max(0.0).min(1.0);
    }

    /// Get the elapsed time since birth in seconds.
    pub fn get_elapsed_seconds(&self) -> f64 {
        self.birth_time.elapsed().as_secs_f64()
    }

    /// Check if the organism has achieved stability (coherence >= 0.7).
    pub fn is_stable(&self) -> bool {
        self.coherence >= 0.7
    }

    /// Get the target interval in milliseconds.
    pub fn get_target_interval_ms(&self) -> f64 {
        self.target_interval_ns as f64 / 1_000_000.0
    }

    /// Get the average actual interval from history in milliseconds.
    pub fn get_avg_interval_ms(&self) -> f64 {
        if self.interval_history.is_empty() {
            return 0.0;
        }
        
        let sum: u64 = self.interval_history.iter().sum();
        let avg_ns = sum / self.interval_history.len() as u64;
        avg_ns as f64 / 1_000_000.0
    }

    /// Get the current jitter in milliseconds.
    pub fn get_jitter_ms(&self) -> f64 {
        let avg_ms = self.get_avg_interval_ms();
        let target_ms = self.get_target_interval_ms();
        (avg_ms - target_ms).abs()
    }

    /// Apply visual stress to metabolism.
    ///
    /// High visual entropy (chaos) degrades coherence.
    /// When the visual field is overwhelming (>0.8 entropy), the organism
    /// experiences physical stress that reduces metabolic coherence.
    pub fn apply_visual_stress(&mut self, visual_entropy: f64) {
        if visual_entropy > 0.8 {
            // Visual overload: reduce coherence by stress amount
            // Stress scales with how far above 0.8 the entropy is
            // Max reduction: 0.05 (when entropy = 1.0)
            let stress = (visual_entropy - 0.8) * 0.25;
            self.coherence = (self.coherence - stress).max(0.0);
        }
    }

    /// Apply audio stress to metabolism.
    ///
    /// Sudden loud noises cause startle response (panic).
    /// Constant high entropy noise drains energy.
    pub fn apply_audio_stress(&mut self, volume: f64, entropy: f64, last_volume: f64) {
        // Startle response: Sudden loud noise (volume > 0.9, was < 0.2)
        if volume > 0.9 && last_volume < 0.2 {
            self.coherence = (self.coherence - 0.3).max(0.0); // PANIC
            log::warn!("🚨 STARTLE RESPONSE: Sudden loud noise detected!");
        }

        // Constant noise drains energy
        if entropy > 0.8 {
            self.coherence = (self.coherence - 0.01).max(0.0);
        }
    }
}

impl Default for Metabolism {
    fn default() -> Self {
        Self::new()
    }
}

