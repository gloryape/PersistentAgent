//! Senses Module
//!
//! Contains the organism's sensory systems:
//! - Bio-Retina: Physical surface for direct energy injection (photons → amplitude)
//! - Bio-Cochlea: Resonant membrane for pressure injection (sound → displacement)
//! - Transducer: Process manager for Python optic_nerve.py

pub mod vision;
pub mod hearing;
pub mod transducer;

// Re-export key types
pub use vision::BioRetina;
pub use hearing::BioCochlea;
pub use transducer::{Transducer, SensoryFrame};

