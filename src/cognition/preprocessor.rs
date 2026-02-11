//! 🔍 Pre-Processor - Feature Extraction
//!
//! Converts raw sensory data (pixels, audio samples) into structured stimuli
//! that can be queued for the Observer's attention.

use crate::AudioAnalysis;
use crate::cognition::stimuli::{Stimulus, StimulusSource, Rect, AudioPattern};
use image::{DynamicImage, Rgb, Rgba};
use imageproc::definitions::Image;

pub struct PreProcessor {
    min_region_size: u32,
    contrast_threshold: u8,
}

impl PreProcessor {
    /// Create a new PreProcessor instance.
    pub fn new() -> Self {
        Self {
            min_region_size: 100, // Minimum 100x100 pixels for interesting regions
            contrast_threshold: 50, // Minimum contrast for interesting regions
        }
    }

    /// Identify features from raw sensory data.
    ///
    /// Returns a list of potential stimuli extracted from visual and audio input.
    ///
    /// If a high-motion region (TV window) is detected, it is prioritized with
    /// high urgency to ensure the Observer focuses on the Nursery Broadcast.
    pub fn identify_features(
        &self,
        visual_frame: &DynamicImage,
        audio_analysis: Option<&AudioAnalysis>,
        high_motion_region: Option<&Rect>,
    ) -> Vec<Stimulus> {
        let mut stimuli = Vec::new();

        // PRIORITY: If high-motion region detected (TV window), create high-priority stimulus
        if let Some(tv_rect) = high_motion_region {
            let tv_entropy = self.calculate_region_entropy(visual_frame, tv_rect);
            let avg_color = self.calculate_average_color(visual_frame, tv_rect);
            
            // High urgency for TV region (motion = attention-grabbing)
            // Motion regions get higher urgency than static regions
            let urgency = (tv_entropy * 0.7 + 0.3).min(1.0); // Boost urgency for motion
            
            let tv_stimulus = Stimulus {
                id: uuid::Uuid::new_v4(),
                source: StimulusSource::VisualRegion {
                    rect: *tv_rect,
                    entropy: tv_entropy,
                    average_color: avg_color,
                },
                urgency,
                novelty: 1.0, // Will be calculated by queue
                salience: urgency, // Temporary, will be recalculated by queue
                timestamp: std::time::Instant::now(),
                attention_count: 0,
            };
            
            stimuli.push(tv_stimulus);
            log::debug!("[NURSERY] TV region detected: {}x{} at ({}, {})", 
                tv_rect.width, tv_rect.height, tv_rect.x, tv_rect.y);
        }

        // Extract other visual regions (high-contrast rectangles)
        let visual_stimuli = self.extract_visual_regions(visual_frame);
        stimuli.extend(visual_stimuli);

        // Extract audio stimuli
        if let Some(audio) = audio_analysis {
            let audio_stimuli = self.extract_audio_stimuli(audio);
            stimuli.extend(audio_stimuli);
        }

        stimuli
    }

    /// Extract interesting visual regions using edge detection and bounding boxes.
    ///
    /// Uses Canny edge detection to find high-contrast regions, then creates
    /// bounding rectangles around them.
    fn extract_visual_regions(&self, frame: &DynamicImage) -> Vec<Stimulus> {
        let gray = frame.to_luma8();
        
        // Run Canny edge detection to find boundaries
        let edges = imageproc::edges::canny(&gray, 50.0, 100.0);
        
        // Find connected components (regions of edges)
        let regions = self.find_connected_regions(&edges);
        
        let mut stimuli = Vec::new();
        
        for region in regions {
            // Filter by size
            if region.width < self.min_region_size || region.height < self.min_region_size {
                continue;
            }
            
            // Calculate entropy for this region
            let region_entropy = self.calculate_region_entropy(frame, &region);
            
            // Calculate average color
            let avg_color = self.calculate_average_color(frame, &region);
            
            // Urgency = entropy (high entropy = more attention-grabbing)
            let urgency = region_entropy.min(1.0);
            
            // Create stimulus
            let stimulus = Stimulus {
                id: uuid::Uuid::new_v4(),
                source: StimulusSource::VisualRegion {
                    rect: region,
                    entropy: region_entropy,
                    average_color: avg_color,
                },
                urgency,
                novelty: 1.0, // Will be calculated by queue
                salience: urgency, // Temporary, will be recalculated by queue
                timestamp: std::time::Instant::now(),
                attention_count: 0,
            };
            
            stimuli.push(stimulus);
        }
        
        stimuli
    }

    /// Find connected regions from edge image.
    ///
    /// Groups edge pixels into connected components and returns bounding rectangles.
    fn find_connected_regions(&self, edges: &image::GrayImage) -> Vec<Rect> {
        let mut regions = Vec::new();
        let width = edges.width();
        let height = edges.height();
        let mut visited = vec![vec![false; height as usize]; width as usize];
        
        // Simple flood fill to find connected edge regions
        for y in 0..height {
            for x in 0..width {
                if edges.get_pixel(x, y)[0] > 0 && !visited[x as usize][y as usize] {
                    // Found a new region, flood fill to find bounds
                    let mut min_x = x;
                    let mut max_x = x;
                    let mut min_y = y;
                    let mut max_y = y;
                    
                    self.flood_fill_region(edges, &mut visited, x, y, width, height, 
                                         &mut min_x, &mut max_x, &mut min_y, &mut max_y);
                    
                    let rect = Rect {
                        x: min_x,
                        y: min_y,
                        width: (max_x - min_x + 1).max(1),
                        height: (max_y - min_y + 1).max(1),
                    };
                    
                    regions.push(rect);
                }
            }
        }
        
        regions
    }

    /// Flood fill to find region bounds.
    fn flood_fill_region(
        &self,
        edges: &image::GrayImage,
        visited: &mut Vec<Vec<bool>>,
        start_x: u32,
        start_y: u32,
        width: u32,
        height: u32,
        min_x: &mut u32,
        max_x: &mut u32,
        min_y: &mut u32,
        max_y: &mut u32,
    ) {
        let mut stack = vec![(start_x, start_y)];
        
        while let Some((x, y)) = stack.pop() {
            if x >= width || y >= height || visited[x as usize][y as usize] {
                continue;
            }
            
            if edges.get_pixel(x, y)[0] == 0 {
                continue;
            }
            
            visited[x as usize][y as usize] = true;
            
            *min_x = (*min_x).min(x);
            *max_x = (*max_x).max(x);
            *min_y = (*min_y).min(y);
            *max_y = (*max_y).max(y);
            
            // Check neighbors
            if x > 0 {
                stack.push((x - 1, y));
            }
            if x < width - 1 {
                stack.push((x + 1, y));
            }
            if y > 0 {
                stack.push((x, y - 1));
            }
            if y < height - 1 {
                stack.push((x, y + 1));
            }
        }
    }

    /// Calculate entropy for a specific region.
    fn calculate_region_entropy(&self, frame: &DynamicImage, rect: &Rect) -> f64 {
        // Extract region by manually copying pixels
        let gray_full = frame.to_luma8();
        let mut region_pixels = Vec::new();
        
        for y in rect.y..(rect.y + rect.height).min(gray_full.height()) {
            for x in rect.x..(rect.x + rect.width).min(gray_full.width()) {
                region_pixels.push(gray_full.get_pixel(x, y)[0]);
            }
        }
        
        // Create a small image from the region
        if region_pixels.is_empty() {
            return 0.0;
        }
        
        let region_img = image::GrayImage::from_raw(rect.width, rect.height, region_pixels)
            .unwrap_or_else(|| image::GrayImage::new(rect.width, rect.height));
        let gray = region_img;
        
        // Run Canny edge detection on the region
        let edges = imageproc::edges::canny(&gray, 50.0, 100.0);
        
        // Count active edge pixels
        let total_pixels = edges.width() * edges.height();
        if total_pixels == 0 {
            return 0.0;
        }
        
        let active_pixels = edges
            .pixels()
            .filter(|p| p[0] > 0)
            .count();
        
        // Entropy = percentage of active pixels
        (active_pixels as f64 / total_pixels as f64).min(1.0)
    }

    /// Calculate average color for a region.
    fn calculate_average_color(&self, frame: &DynamicImage, rect: &Rect) -> Rgb<u8> {
        let rgb_frame = frame.to_rgb8();
        
        let mut r_sum = 0u64;
        let mut g_sum = 0u64;
        let mut b_sum = 0u64;
        let mut count = 0u64;
        
        for y in rect.y..(rect.y + rect.height).min(rgb_frame.height()) {
            for x in rect.x..(rect.x + rect.width).min(rgb_frame.width()) {
                let pixel = rgb_frame.get_pixel(x, y);
                r_sum += pixel[0] as u64;
                g_sum += pixel[1] as u64;
                b_sum += pixel[2] as u64;
                count += 1;
            }
        }
        
        if count > 0 {
            Rgb([
                (r_sum / count) as u8,
                (g_sum / count) as u8,
                (b_sum / count) as u8,
            ])
        } else {
            Rgb([128, 128, 128]) // Default gray
        }
    }

    /// Extract audio stimuli from audio analysis.
    fn extract_audio_stimuli(&self, audio: &AudioAnalysis) -> Vec<Stimulus> {
        let mut stimuli = Vec::new();
        
        // Only create audio stimulus if volume is significant
        if audio.volume > 0.1 {
            let pattern = AudioPattern {
                dominant_freq: audio.dominant_freq,
                harmonic_ratio: 1.0 - audio.entropy, // Low entropy = harmonic
                is_speech_like: self.detect_speech_like(audio),
            };
            
            let urgency = audio.volume; // Urgency = volume
            
            let stimulus = Stimulus {
                id: uuid::Uuid::new_v4(),
                source: StimulusSource::AudioStream {
                    frequency: audio.dominant_freq,
                    pattern,
                    volume: audio.volume,
                },
                urgency,
                novelty: 1.0, // Will be calculated by queue
                salience: urgency, // Temporary
                timestamp: std::time::Instant::now(),
                attention_count: 0,
            };
            
            stimuli.push(stimulus);
        }
        
        stimuli
    }

    /// Detect if audio pattern is speech-like.
    ///
    /// Simple heuristic: speech typically has frequencies in 85-255 Hz range
    /// and moderate entropy (not pure tone, not white noise).
    fn detect_speech_like(&self, audio: &AudioAnalysis) -> bool {
        let freq = audio.dominant_freq;
        let entropy = audio.entropy;
        
        // Speech-like: frequency in human voice range, moderate entropy
        freq >= 85.0 && freq <= 255.0 && entropy > 0.3 && entropy < 0.7
    }
}

impl Default for PreProcessor {
    fn default() -> Self {
        Self::new()
    }
}

