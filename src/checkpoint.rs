//! Full simulation save/load (Sanctuary + minimal organism state).
//!
//! Saves to `data/saves/{name}_{timestamp}.qsim` as bincode.

use std::error::Error;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

use bincode;
use serde::{Deserialize, Serialize};

use crate::cognition::sanctuary::{Sanctuary, SanctuaryCheckpoint};
use crate::cognition::sanctuary::DEFAULT_DATA_DIR;

/// Full checkpoint: sanctuary + organism state for named saves.
#[derive(Serialize, Deserialize)]
pub struct FullCheckpoint {
    pub sanctuary: SanctuaryCheckpoint,
    pub organism: OrganismCheckpoint,
    pub save_name: String,
    pub save_timestamp: String,
    pub version: String,
}

/// Minimal organism state (agent position, phase, coherence, tick).
#[derive(Clone, Serialize, Deserialize)]
pub struct OrganismCheckpoint {
    pub agent_position_pixels: (f32, f32),
    pub agent_phase: f32,
    pub coherence: f64,
    pub tick_count: u32,
}

/// Save current simulation to `data/saves/{name}_{timestamp}.qsim`.
pub fn save_simulation(
    sanctuary: &Sanctuary,
    agent_position: (f32, f32),
    agent_phase: f32,
    coherence: f64,
    tick_count: u32,
    name: &str,
) -> Result<PathBuf, Box<dyn Error>> {
    let saves_dir = PathBuf::from(DEFAULT_DATA_DIR).join("saves");
    fs::create_dir_all(&saves_dir)?;

    let timestamp = chrono::Utc::now().format("%Y-%m-%d_%H%M%S");
    let filename = format!("{}_{}.qsim", name, timestamp);
    let save_path = saves_dir.join(&filename);

    let checkpoint = FullCheckpoint {
        sanctuary: sanctuary.to_checkpoint(),
        organism: OrganismCheckpoint {
            agent_position_pixels: agent_position,
            agent_phase,
            coherence,
            tick_count,
        },
        save_name: name.to_string(),
        save_timestamp: timestamp.to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    };

    let file = File::create(&save_path)?;
    let mut writer = BufWriter::new(file);
    bincode::serialize_into(&mut writer, &checkpoint)?;
    writer.flush()?;

    log::info!("Simulation saved: {}", save_path.display());
    Ok(save_path)
}

/// Load simulation from a .qsim file. Returns sanctuary and organism state.
pub fn load_simulation(path: &Path) -> Result<(Sanctuary, OrganismCheckpoint), Box<dyn Error>> {
    if !path.exists() {
        return Err(format!("Save file not found: {}", path.display()).into());
    }

    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let checkpoint: FullCheckpoint = bincode::deserialize_from(reader)
        .map_err(|e| format!("Deserialization error: {}", e))?;

    if checkpoint.version != env!("CARGO_PKG_VERSION") {
        log::warn!(
            "Loading from different version: {} (current: {})",
            checkpoint.version,
            env!("CARGO_PKG_VERSION")
        );
    }

    let data_dir = PathBuf::from(DEFAULT_DATA_DIR);
    let sanctuary = Sanctuary::from_checkpoint(checkpoint.sanctuary, data_dir)?;

    log::info!(
        "Simulation loaded: {} (saved: {})",
        checkpoint.save_name,
        checkpoint.save_timestamp
    );

    Ok((sanctuary, checkpoint.organism))
}
