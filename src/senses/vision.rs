//! 👁️ Bio-Retina - Physical Surface for Bidirectional Field Perception
//!
//! The Bio-Retina is a 2D grid of receptors that can both:
//! 1. **Inject** external energy into the Sanctuary field (photons → amplitude + phase)
//! 2. **Sample** the Sanctuary field around the organism's position (proprioception)
//!
//! Proprioceptive mode gives the organism sight of its own environment:
//! - Amplitude → Brightness (how much energy is at each voxel)
//! - Phase → Hue (what direction/intention was deposited there)
//!
//! The organism literally sees its own trail, its own terrain, and the field
//! decaying around it. This closes the Reflection Axiom feedback loop.

use crate::cognition::sanctuary::Sanctuary;

/// Energy scale factor for photon-to-amplitude conversion
/// Higher values = brighter light = more energy injected
const PHOTON_ENERGY_SCALE: f32 = 0.01;

/// Maximum amplitude for normalization when sampling the field
/// Field amplitudes above this are clamped to full brightness
const MAX_VISIBLE_AMPLITUDE: f32 = 2.0;

/// A single photoreceptor in the retina grid
#[derive(Debug, Clone)]
struct Receptor {
    /// X coordinate in Sanctuary field (relative to grid origin)
    x: i32,
    /// Y coordinate in Sanctuary field (relative to grid origin)
    y: i32,
    /// Last brightness value (for adaptation)
    last_brightness: f32,
    /// Adaptation level (prevents saturation)
    adaptation_level: f32,
}

/// The Bio-Retina - A physical surface for bidirectional field perception
pub struct BioRetina {
    /// Resolution of the receptor grid (64x64)
    resolution: (u32, u32),
    /// 2D grid of receptors, each mapping to a Sanctuary voxel
    receptor_grid: Vec<Vec<Receptor>>,
}

impl BioRetina {
    /// Create a new Bio-Retina with the specified resolution
    ///
    /// # Arguments
    /// * `width` - Grid width (default: 64)
    /// * `height` - Grid height (default: 64)
    pub fn new(width: u32, height: u32) -> Self {
        let mut receptor_grid = Vec::with_capacity(height as usize);
        
        for y in 0..height {
            let mut row = Vec::with_capacity(width as usize);
            for x in 0..width {
                row.push(Receptor {
                    x: x as i32,
                    y: y as i32,
                    last_brightness: 0.0,
                    adaptation_level: 1.0,
                });
            }
            receptor_grid.push(row);
        }
        
        log::info!("👁️ Bio-Retina initialized: {}x{} receptor grid", width, height);
        
        Self {
            resolution: (width, height),
            receptor_grid,
        }
    }

    /// Create a Bio-Retina with default 64x64 resolution
    pub fn default() -> Self {
        Self::new(64, 64)
    }

    /// Inject RGB grid data into the Sanctuary field
    ///
    /// # Arguments
    /// * `rgb_grid` - 2D grid of RGB pixels: `Vec<Vec<[u8; 3]>>` where each pixel is [R, G, B]
    /// * `sanctuary` - Reference to the Sanctuary field to inject into
    ///
    /// # Physics
    /// - **Brightness (Luminance)** → **Amplitude**: `L = 0.299*R + 0.587*G + 0.114*B`
    /// - **Hue** → **Phase**: Red (0°) → 0.0 rad, Green (120°) → 2.09 rad, Blue (240°) → 4.19 rad
    ///
    /// The organism "sees" because photons physically increase voxel amplitudes.
    pub fn inject_into_sanctuary(
        &mut self,
        rgb_grid: &[Vec<[u8; 3]>],
        sanctuary: &mut Sanctuary,
    ) {
        let (width, height) = self.resolution;
        
        // Ensure grid dimensions match
        if rgb_grid.len() != height as usize {
            log::warn!("RGB grid height mismatch: expected {}, got {}", height, rgb_grid.len());
            return;
        }
        
        // Process each receptor
        for (y, row) in self.receptor_grid.iter_mut().enumerate() {
            if y >= rgb_grid.len() {
                break;
            }
            
            let rgb_row = &rgb_grid[y];
            if rgb_row.len() != width as usize {
                log::warn!("RGB grid width mismatch at row {}: expected {}, got {}", y, width, rgb_row.len());
                continue;
            }
            
            for (x, receptor) in row.iter_mut().enumerate() {
                if x >= rgb_row.len() {
                    break;
                }
                
                let [r, g, b] = rgb_row[x];
                
                // Convert RGB to Brightness (Luminance)
                // Standard ITU-R BT.601 formula
                let brightness = (0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32) / 255.0;
                
                // Convert Brightness to Energy Amplitude
                // Apply adaptation to prevent saturation
                let adapted_brightness = brightness * receptor.adaptation_level;
                let amplitude = adapted_brightness * PHOTON_ENERGY_SCALE;
                
                // Convert RGB to Hue (Phase)
                let phase = Self::rgb_to_phase(r, g, b);
                
                // Inject energy into Sanctuary at receptor's coordinates
                sanctuary.inject_energy(receptor.x, receptor.y, 0, amplitude, phase);
                
                // Update receptor state (for adaptation)
                receptor.last_brightness = brightness;
                
                // Adaptive gain control: reduce sensitivity if too bright
                if brightness > 0.8 {
                    receptor.adaptation_level = (receptor.adaptation_level * 0.99).max(0.1);
                } else if brightness < 0.2 {
                    receptor.adaptation_level = (receptor.adaptation_level * 1.01).min(1.0);
                }
            }
        }
    }

    /// Sample the Sanctuary field around the organism's position (Proprioception)
    ///
    /// This is the inverse of `inject_into_sanctuary`: instead of writing external
    /// pixels INTO the field, it READS the field state and converts it to an RGB
    /// perceptual frame that the organism "sees."
    ///
    /// # Arguments
    /// * `sanctuary` - Reference to the Sanctuary field to sample from
    /// * `center_voxel` - The organism's current voxel position (center of view)
    ///
    /// # Returns
    /// * `Vec<Vec<[u8; 3]>>` - A 2D RGB grid representing the organism's perception
    ///   of its surrounding field. Amplitude → Brightness, Phase → Hue.
    ///
    /// # Physics
    /// - **Amplitude → Brightness**: High energy voxels appear bright, void is dark
    /// - **Phase → Hue**: The deposited intention/direction becomes visible color
    /// - The organism sees its own trail (bright, colored), unexplored void (black),
    ///   and decaying territory (dimming)
    pub fn sample_from_sanctuary(
        &self,
        sanctuary: &Sanctuary,
        center_voxel: (i32, i32),
    ) -> Vec<Vec<[u8; 3]>> {
        let (width, height) = self.resolution;
        let half_w = width as i32 / 2;
        let half_h = height as i32 / 2;

        let mut rgb_grid = Vec::with_capacity(height as usize);

        for gy in 0..height as i32 {
            let mut row = Vec::with_capacity(width as usize);
            for gx in 0..width as i32 {
                // Map grid position to field voxel (centered on organism)
                let vx = center_voxel.0 + (gx - half_w);
                let vy = center_voxel.1 + (gy - half_h);

                let amplitude = sanctuary.density_at((vx, vy, 0));
                let phase = sanctuary.phase_at((vx, vy, 0));

                // Amplitude → Brightness [0.0, 1.0]
                let brightness = (amplitude / MAX_VISIBLE_AMPLITUDE).min(1.0);

                // Phase → Hue, then combine with brightness to get RGB
                let rgb = Self::phase_brightness_to_rgb(phase, brightness);
                row.push(rgb);
            }
            rgb_grid.push(row);
        }

        rgb_grid
    }

    /// Convert phase angle + brightness into an RGB pixel
    ///
    /// This is the inverse of the injection encoding:
    /// - Phase (0 to 2π) → Hue (0° to 360°)
    /// - Brightness (0 to 1) → Value in HSV
    /// - Saturation is 1.0 when there's energy, 0.0 when void
    fn phase_brightness_to_rgb(phase: f32, brightness: f32) -> [u8; 3] {
        if brightness < 0.001 {
            return [0, 0, 0]; // Void is black
        }

        // Phase → Hue (radians to degrees)
        let hue = (phase.rem_euclid(2.0 * std::f32::consts::PI) * 180.0
            / std::f32::consts::PI)
            .min(360.0);

        // HSV to RGB (S=1.0 for saturated color, V=brightness)
        let s = 1.0f32;
        let v = brightness;
        let c = v * s;
        let h_prime = hue / 60.0;
        let x = c * (1.0 - (h_prime % 2.0 - 1.0).abs());
        let m = v - c;

        let (r1, g1, b1) = if h_prime < 1.0 {
            (c, x, 0.0)
        } else if h_prime < 2.0 {
            (x, c, 0.0)
        } else if h_prime < 3.0 {
            (0.0, c, x)
        } else if h_prime < 4.0 {
            (0.0, x, c)
        } else if h_prime < 5.0 {
            (x, 0.0, c)
        } else {
            (c, 0.0, x)
        };

        [
            ((r1 + m) * 255.0) as u8,
            ((g1 + m) * 255.0) as u8,
            ((b1 + m) * 255.0) as u8,
        ]
    }

    /// Convert RGB values to phase angle in radians
    ///
    /// # Mapping
    /// - Red (hue 0°) → phase 0.0 rad
    /// - Green (hue 120°) → phase 2.09 rad (2π/3)
    /// - Blue (hue 240°) → phase 4.19 rad (4π/3 ≈ π)
    ///
    /// Uses HSV color space conversion for accurate hue calculation.
    fn rgb_to_phase(r: u8, g: u8, b: u8) -> f32 {
        let r_norm = r as f32 / 255.0;
        let g_norm = g as f32 / 255.0;
        let b_norm = b as f32 / 255.0;
        
        let max = r_norm.max(g_norm).max(b_norm);
        let min = r_norm.min(g_norm).min(b_norm);
        let delta = max - min;
        
        if delta == 0.0 {
            // Grayscale - no hue, use neutral phase
            return 0.0;
        }
        
        let hue = if max == r_norm {
            // Red is max
            let segment = ((g_norm - b_norm) / delta) % 6.0;
            segment * 60.0
        } else if max == g_norm {
            // Green is max
            let segment = (b_norm - r_norm) / delta + 2.0;
            segment * 60.0
        } else {
            // Blue is max
            let segment = (r_norm - g_norm) / delta + 4.0;
            segment * 60.0
        };
        
        // Convert hue (0-360°) to phase (0-2π radians)
        let phase = (hue * std::f32::consts::PI / 180.0) % (2.0 * std::f32::consts::PI);
        phase
    }

    /// Get the resolution of the receptor grid
    pub fn resolution(&self) -> (u32, u32) {
        self.resolution
    }

    /// Get the number of receptors
    pub fn receptor_count(&self) -> usize {
        self.resolution.0 as usize * self.resolution.1 as usize
    }
}

impl Default for BioRetina {
    fn default() -> Self {
        Self::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rgb_to_phase() {
        // Red should map to ~0.0
        let phase_red = BioRetina::rgb_to_phase(255, 0, 0);
        assert!(phase_red < 0.1 || phase_red > 6.0, "Red should map to ~0.0 or ~2π");
        
        // Green should map to ~2.09 (2π/3)
        let phase_green = BioRetina::rgb_to_phase(0, 255, 0);
        assert!((phase_green - 2.09).abs() < 0.5, "Green should map to ~2.09 rad");
        
        // Blue should map to ~4.19 (4π/3)
        let phase_blue = BioRetina::rgb_to_phase(0, 0, 255);
        assert!((phase_blue - 4.19).abs() < 0.5, "Blue should map to ~4.19 rad");
    }

    #[test]
    fn test_retina_creation() {
        let retina = BioRetina::new(64, 64);
        assert_eq!(retina.resolution(), (64, 64));
        assert_eq!(retina.receptor_count(), 64 * 64);
    }

    #[test]
    fn test_phase_brightness_roundtrip() {
        // Bright red (phase ~0, brightness 1.0) should produce red-ish pixel
        let rgb = BioRetina::phase_brightness_to_rgb(0.0, 1.0);
        assert!(rgb[0] > 200, "Red channel should be high for phase=0");
        
        // Void should be black
        let rgb_void = BioRetina::phase_brightness_to_rgb(0.0, 0.0);
        assert_eq!(rgb_void, [0, 0, 0], "Void should be black");
        
        // Green phase (~2.09 rad) should produce green-ish
        let rgb_green = BioRetina::phase_brightness_to_rgb(2.09, 0.8);
        assert!(rgb_green[1] > rgb_green[0] && rgb_green[1] > rgb_green[2],
            "Green channel should dominate at phase ~2.09");
    }

    #[test]
    fn test_sample_from_empty_sanctuary() {
        let retina = BioRetina::new(8, 8);
        let sanctuary = Sanctuary::new();
        let grid = retina.sample_from_sanctuary(&sanctuary, (50, 50));
        
        // Empty sanctuary should produce all-black grid
        assert_eq!(grid.len(), 8);
        assert_eq!(grid[0].len(), 8);
        for row in &grid {
            for pixel in row {
                assert_eq!(*pixel, [0, 0, 0], "Empty field should be black");
            }
        }
    }
}
