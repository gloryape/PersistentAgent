//! Transducer - Reads sensory data from stdin (SENS protocol)
//!
//! Parses binary SENS chunks from stdin:
//! [HEADER: 4 bytes "SENS"]
//! [TYPE:   1 byte (1=Video, 2=Audio)]
//! [SIZE:   4 bytes u32 LE]
//! [PAYLOAD: size bytes]
//!
//! One frame = Video chunk (type 1) + Audio chunk (type 2)

use std::io::{BufReader, Read};
use std::error::Error;

/// A sensory frame containing RGB grid and audio samples
#[derive(Debug, Clone)]
pub struct SensoryFrame {
    /// RGB grid: 64x64 pixels, each pixel is [R, G, B]
    pub rgb_grid: Vec<Vec<[u8; 3]>>,
    /// Audio samples: 2048 float32 samples in range [-1.0, 1.0]
    pub audio_samples: Vec<f32>,
    /// Frame timestamp (tick number when received)
    pub timestamp: u64,
}

/// Transducer reads SENS protocol from stdin
pub struct Transducer {
    reader: BufReader<std::io::Stdin>,
    frame_counter: u64,
    eof_reached: bool,
}

impl Transducer {
    /// Create a transducer that reads from stdin
    pub fn from_stdin() -> Self {
        Self {
            reader: BufReader::new(std::io::stdin()),
            frame_counter: 0,
            eof_reached: false,
        }
    }

    /// Read a single frame from stdin (SENS protocol)
    ///
    /// Returns Ok(SensoryFrame) if a complete frame (Video + Audio) was read.
    /// Returns Err if EOF or incomplete data (caller should handle as "no frame available").
    pub fn read_frame(&mut self) -> Result<SensoryFrame, Box<dyn Error>> {
        if self.eof_reached {
            return Err("EOF reached".into());
        }

        // Read Video chunk (type 1)
        let rgb_payload = self.read_sens_chunk(1)?;
        let rgb_grid = self.parse_rgb_payload(rgb_payload)?;
        
        // Read Audio chunk (type 2)
        let audio_payload = self.read_sens_chunk(2)?;
        let audio_samples = self.parse_audio_payload(audio_payload)?;
        
        self.frame_counter += 1;
        
        Ok(SensoryFrame {
            rgb_grid,
            audio_samples,
            timestamp: self.frame_counter,
        })
    }
    
    /// Read a single SENS chunk and return payload
    fn read_sens_chunk(&mut self, expected_type: u8) -> Result<Vec<u8>, Box<dyn Error>> {
        // Read header (4 bytes)
        let mut header = [0u8; 4];
        match self.reader.read_exact(&mut header) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                self.eof_reached = true;
                return Err("EOF while reading header".into());
            }
            Err(e) => return Err(e.into()),
        }
        
        if &header != b"SENS" {
            return Err(format!("Invalid chunk header: {:?}, expected SENS", header).into());
        }
        
        // Read type byte
        let mut type_byte = [0u8; 1];
        self.reader.read_exact(&mut type_byte)?;
        if type_byte[0] != expected_type {
            return Err(format!("Unexpected chunk type: {}, expected {}", type_byte[0], expected_type).into());
        }
        
        // Read size (u32 LE)
        let mut size_bytes = [0u8; 4];
        self.reader.read_exact(&mut size_bytes)?;
        let size = u32::from_le_bytes(size_bytes) as usize;
        
        // Read payload
        let mut payload = vec![0u8; size];
        match self.reader.read_exact(&mut payload) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                self.eof_reached = true;
                return Err("EOF while reading payload".into());
            }
            Err(e) => return Err(e.into()),
        }
        
        Ok(payload)
    }
    
    /// Parse RGB payload (12288 bytes) into 64x64x3 grid
    fn parse_rgb_payload(&self, payload: Vec<u8>) -> Result<Vec<Vec<[u8; 3]>>, Box<dyn Error>> {
        const WIDTH: usize = 64;
        const HEIGHT: usize = 64;
        const EXPECTED_SIZE: usize = WIDTH * HEIGHT * 3;
        
        if payload.len() != EXPECTED_SIZE {
            return Err(format!("Invalid RGB payload size: {}, expected {}", payload.len(), EXPECTED_SIZE).into());
        }
        
        let mut rgb_grid = Vec::with_capacity(HEIGHT);
        for y in 0..HEIGHT {
            let mut row = Vec::with_capacity(WIDTH);
            for x in 0..WIDTH {
                let idx = (y * WIDTH + x) * 3;
                row.push([payload[idx], payload[idx + 1], payload[idx + 2]]);
            }
            rgb_grid.push(row);
        }
        Ok(rgb_grid)
    }
    
    /// Parse Audio payload (8192 bytes) into 2048 float32 samples
    fn parse_audio_payload(&self, payload: Vec<u8>) -> Result<Vec<f32>, Box<dyn Error>> {
        const EXPECTED_SIZE: usize = 2048 * 4;
        
        if payload.len() != EXPECTED_SIZE {
            return Err(format!("Invalid audio payload size: {}, expected {}", payload.len(), EXPECTED_SIZE).into());
        }
        
        let mut audio_samples = Vec::with_capacity(2048);
        for chunk in payload.chunks_exact(4) {
            let sample = f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
            audio_samples.push(sample);
        }
        Ok(audio_samples)
    }

    /// Check if stdin is still available (not EOF)
    pub fn is_alive(&self) -> bool {
        !self.eof_reached
    }

    /// Get the number of frames read so far
    pub fn frame_count(&self) -> u64 {
        self.frame_counter
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_parsing() {
        // This would require a mock Python process
        // For now, just verify the struct sizes
        let frame = SensoryFrame {
            rgb_grid: vec![vec![[255, 0, 0]; 64]; 64],
            audio_samples: vec![0.0; 2048],
            timestamp: 0,
        };
        
        assert_eq!(frame.rgb_grid.len(), 64);
        assert_eq!(frame.rgb_grid[0].len(), 64);
        assert_eq!(frame.audio_samples.len(), 2048);
    }
}


