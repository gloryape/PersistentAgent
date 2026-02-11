//! 👂 Bio-Cochlea - Resonant Membrane for Direct Pressure Injection
//!
//! The Bio-Cochlea is a physical membrane at a specific location in the Sanctuary.
//! Sound pressure waves directly displace the membrane amplitude. This creates
//! natural ripple propagation through the field. This is "Literal Perception" -
//! no FFT analysis, just physical displacement.

use std::error::Error;
use crate::cognition::sanctuary::Sanctuary;

/// Audio analysis results (legacy type for observability compatibility)
#[derive(Debug, Clone)]
pub struct AudioAnalysis {
    pub volume: f64,           // RMS amplitude (0.0-1.0)
    pub entropy: f64,         // Audio entropy (0.0-1.0)
    pub dominant_freq: f64,   // Dominant frequency in Hz
    pub spectrum: Vec<f64>,    // Frequency spectrum magnitudes
}

/// Membrane radius in voxels (how many voxels the "drum" occupies)
const MEMBRANE_RADIUS: i32 = 5;

/// Energy scale factor for audio displacement
/// Higher values = louder sound = more displacement
const PRESSURE_ENERGY_SCALE: f32 = 0.05;

/// The Bio-Cochlea - A resonant membrane that injects pressure into the Sanctuary
pub struct BioCochlea {
    /// Center location of the membrane in Sanctuary coordinates
    drum_location: (i32, i32, i32),
    /// Radius of the membrane (in voxels)
    membrane_radius: i32,
    /// Last displacement value (for smoothing)
    last_displacement: f32,
}

impl BioCochlea {
    /// Create a new Bio-Cochlea with membrane at the specified location
    ///
    /// # Arguments
    /// * `center_x`, `center_y`, `center_z` - Center coordinates of the membrane
    /// * `radius` - Radius of the membrane in voxels (default: 5)
    pub fn new(center_x: i32, center_y: i32, center_z: i32, radius: i32) -> Self {
        log::info!(
            "👂 Bio-Cochlea initialized: membrane at ({}, {}, {}) with radius {}",
            center_x, center_y, center_z, radius
        );
        
        Self {
            drum_location: (center_x, center_y, center_z),
            membrane_radius: radius,
            last_displacement: 0.0,
        }
    }

    /// Create a Bio-Cochlea at the center of a 64x64 grid
    pub fn default() -> Self {
        Self::new(32, 32, 0, MEMBRANE_RADIUS)
    }

    /// Inject audio samples into the Sanctuary field
    ///
    /// # Arguments
    /// * `audio_samples` - Vector of float32 samples in range [-1.0, 1.0]
    /// * `sanctuary` - Reference to the Sanctuary field to inject into
    ///
    /// # Physics
    /// - **PCM Sample Value** → **Displacement**: Positive = push (add energy), Negative = pull (subtract energy, min 0)
    /// - **Spatial Distribution**: Samples are distributed across membrane voxels
    /// - **Ripple Propagation**: The Sanctuary's existing physics handles natural wave propagation
    ///
    /// The organism "hears" because sound pressure physically deforms the membrane.
    pub fn inject_into_sanctuary(
        &mut self,
        audio_samples: &[f32],
        sanctuary: &mut Sanctuary,
    ) {
        if audio_samples.is_empty() {
            return;
        }
        
        // Calculate average displacement from samples (RMS-like)
        let sum_squares: f32 = audio_samples.iter().map(|&s| s * s).sum();
        let rms = (sum_squares / audio_samples.len() as f32).sqrt();
        
        // Apply smoothing to prevent abrupt changes
        let smoothed_displacement = self.last_displacement * 0.7 + rms * 0.3;
        self.last_displacement = smoothed_displacement;
        
        // Convert RMS to energy displacement
        let base_energy = smoothed_displacement * PRESSURE_ENERGY_SCALE;
        
        // Distribute energy across membrane voxels
        // Use a circular distribution with falloff from center
        let (center_x, center_y, center_z) = self.drum_location;
        
        for dy in -self.membrane_radius..=self.membrane_radius {
            for dx in -self.membrane_radius..=self.membrane_radius {
                let distance_sq = (dx * dx + dy * dy) as f32;
                let radius_sq = (self.membrane_radius * self.membrane_radius) as f32;
                
                // Only affect voxels within membrane radius
                if distance_sq > radius_sq {
                    continue;
                }
                
                // Calculate falloff factor (Gaussian-like)
                // Voxels at center get full energy, edge voxels get less
                let falloff = 1.0 - (distance_sq / radius_sq);
                let energy = base_energy * falloff;
                
                // Calculate phase from sample pattern
                // Use the dominant frequency component (simplified: use average sample sign)
                let avg_sign = if smoothed_displacement > 0.0 { 1.0 } else { -1.0 };
                let phase = if avg_sign > 0.0 {
                    0.0  // Positive pressure
                } else {
                    std::f32::consts::PI  // Negative pressure (rarefaction)
                };
                
                // Inject energy at this voxel
                let voxel_x = center_x + dx;
                let voxel_y = center_y + dy;
                sanctuary.inject_energy(voxel_x, voxel_y, center_z, energy, phase);
            }
        }
        
        // Also inject a strong pulse at the center for loud sounds
        if smoothed_displacement > 0.5 {
            let center_energy = base_energy * 2.0;  // Double energy at center
            sanctuary.inject_energy(center_x, center_y, center_z, center_energy, 0.0);
        }
    }

    /// Get the membrane center location
    pub fn drum_location(&self) -> (i32, i32, i32) {
        self.drum_location
    }

    /// Get the membrane radius
    pub fn membrane_radius(&self) -> i32 {
        self.membrane_radius
    }

    /// Set a new membrane location (for dynamic repositioning)
    pub fn set_drum_location(&mut self, x: i32, y: i32, z: i32) {
        self.drum_location = (x, y, z);
        log::debug!("Bio-Cochlea membrane moved to ({}, {}, {})", x, y, z);
    }
}

impl Default for BioCochlea {
    fn default() -> Self {
        Self::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cochlea_creation() {
        let cochlea = BioCochlea::new(32, 32, 0, 5);
        assert_eq!(cochlea.drum_location(), (32, 32, 0));
        assert_eq!(cochlea.membrane_radius(), 5);
    }

    #[test]
    fn test_cochlea_default() {
        let cochlea = BioCochlea::default();
        assert_eq!(cochlea.drum_location(), (32, 32, 0));
    }
}
