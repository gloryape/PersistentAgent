//! 🏛️ Sanctuary - 4D Scalar Field Environment with Causal Memory
//!
//! The Sanctuary is NOT a standard game environment. It is a simulation of
//! Dynamic Vacuum Field Theory (DVFT) derived from persistence axioms.
//!
//! ## Core Theory: Morality is Thermodynamic Efficiency
//!
//! **Exclusion Axiom**: The vacuum has "Stiffness." Pushing too much energy
//! into one spot creates infinite resistance: V(ρ) → ∞
//!
//! **Reflection Axiom**: The vacuum has "Memory." Stability requires a feedback
//! loop from the past. An agent's action is only efficient if it resonates
//! (phase-locks) with the history of that location.
//!
//! ## Mathematical Foundation
//!
//! - **Stiffness**: R = K₀ · ρ² (K₀ ≈ 2.0)
//! - **Resonance**: Σ cos(θ_in - θ_history) weighted by recency
//! - **Efficiency**: E_effective = (E_in - R) × (1 + γ × Resonance) (γ ≈ 0.8)
//!
//! ## Dual-Stream Data Persistence
//!
//! - **Stream A (Metrics)**: High-frequency physics data in Parquet format
//! - **Stream B (Logs)**: Agent state changes in JSON Lines (.jsonl) format
//!
//! The data proves: **Coherence = Survival**

use std::collections::{HashMap, VecDeque};
use std::fs::{self, File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Write};
use std::error::Error;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use chrono::Utc;
use polars::prelude::*;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use bincode;

// ═══════════════════════════════════════════════════════════════════════════
// CONSTANTS: The Physics of the Vacuum
// ═══════════════════════════════════════════════════════════════════════════

/// Depth of the causal memory buffer (how far back the vacuum "remembers")
pub const MEMORY_DEPTH: usize = 64;

/// Stiffness constant K₀ (Exclusion Axiom coefficient)
/// Lower value (was 2.0) — at K₀=2.0, even amplitude 0.5 produces stiffness 0.5
/// which consumes ALL babbling energy (0.5). At K₀=0.5, stiffness at amplitude 0.5
/// is only 0.125, leaving room for effective energy deposition.
/// The game must be winnable before the organism can discover that coherent
/// behavior is rewarded.
pub const STIFFNESS_K0: f32 = 0.5;

/// Resonance coupling constant γ
/// Increased (was 0.8) — stronger reward for phase-consistent behavior.
/// At γ=1.5, perfect resonance (1.0) gives a 2.5x multiplier on base energy,
/// making coherent revisits dramatically more efficient than random ones.
pub const RESONANCE_GAMMA: f32 = 1.5;

/// Entropy decay rate D
/// Reduced (was 0.05) — at 0.05/tick, a voxel with amplitude 0.5 vanishes in
/// 10 ticks. The organism visits one voxel every 15 ticks (babbling rate).
/// It could never revisit fast enough. At 0.01/tick, the same voxel lasts 50
/// ticks, giving the organism time to build persistent trails.
pub const ENTROPY_DECAY: f32 = 0.01;

/// Minimum amplitude before a voxel is considered "empty"
pub const MIN_AMPLITUDE: f32 = 0.001;

// ═══════════════════════════════════════════════════════════════════════════
// CONSTANTS: Data Persistence
// ═══════════════════════════════════════════════════════════════════════════

/// Number of records to buffer before flushing to Parquet
pub const METRICS_BUFFER_SIZE: usize = 1_000;

/// Number of log entries before flushing to JSONL
pub const LOGS_BUFFER_SIZE: usize = 100;

/// Default data directory
pub const DEFAULT_DATA_DIR: &str = "data";

// ═══════════════════════════════════════════════════════════════════════════
// DATA STRUCTURES: Field Physics
// ═══════════════════════════════════════════════════════════════════════════

/// A snapshot of field state at a moment in time
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct FieldState {
    /// Energy amplitude at this point (ρ)
    pub amplitude: f32,
    /// Phase/intention angle (θ) in radians
    pub phase: f32,
    /// When this state was recorded (tick number)
    pub timestamp: u64,
}

impl FieldState {
    /// Create a new field state
    pub fn new(amplitude: f32, phase: f32, timestamp: u64) -> Self {
        Self {
            amplitude,
            phase,
            timestamp,
        }
    }

    /// Create an empty/vacuum state
    pub fn vacuum(timestamp: u64) -> Self {
        Self {
            amplitude: 0.0,
            phase: 0.0,
            timestamp,
        }
    }
}

impl Default for FieldState {
    fn default() -> Self {
        Self::vacuum(0)
    }
}

/// A voxel in the scalar field with causal memory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Voxel {
    /// Current field state
    pub current: FieldState,
    /// Historical states (circular buffer, newest at front)
    pub history: VecDeque<FieldState>,
}

impl Voxel {
    /// Create a new empty voxel
    pub fn new() -> Self {
        Self {
            current: FieldState::default(),
            history: VecDeque::with_capacity(MEMORY_DEPTH),
        }
    }

    /// Push current state to history and update with new state
    pub fn update(&mut self, new_state: FieldState) {
        // Push current to history (front = newest)
        self.history.push_front(self.current);
        
        // Maintain fixed buffer size (circular)
        while self.history.len() > MEMORY_DEPTH {
            self.history.pop_back();
        }
        
        // Update current
        self.current = new_state;
    }

    /// Apply entropy decay to amplitude
    pub fn decay(&mut self, decay_rate: f32) {
        self.current.amplitude = (self.current.amplitude - decay_rate).max(0.0);
    }

    /// Calculate resonance with a given input phase
    /// Returns a value in [-1.0, 1.0], where 1.0 = perfect alignment
    pub fn calculate_resonance(&self, input_phase: f32) -> f32 {
        if self.history.is_empty() {
            return 0.0; // No history = no resonance
        }

        let mut weighted_sum = 0.0f32;
        let mut weight_total = 0.0f32;

        // Iterate through history, weighting recent memories higher
        for (i, state) in self.history.iter().enumerate() {
            // Recency weight: exponential decay (1.0 for newest, decaying for older)
            let recency_weight = (-(i as f32) / (MEMORY_DEPTH as f32 / 2.0)).exp();
            
            // Amplitude weight: stronger memories count more
            let amplitude_weight = state.amplitude.max(0.1);
            
            // Combined weight
            let weight = recency_weight * amplitude_weight;
            
            // Phase alignment: cos(θ_in - θ_history)
            let phase_diff = input_phase - state.phase;
            let alignment = phase_diff.cos();
            
            weighted_sum += alignment * weight;
            weight_total += weight;
        }

        if weight_total > 0.0 {
            weighted_sum / weight_total
        } else {
            0.0
        }
    }

    /// Calculate stiffness/resistance based on current density
    /// R = K₀ · ρ²
    pub fn calculate_stiffness(&self) -> f32 {
        STIFFNESS_K0 * self.current.amplitude.powi(2)
    }

    /// Check if this voxel is effectively empty
    pub fn is_empty(&self) -> bool {
        self.current.amplitude < MIN_AMPLITUDE && self.history.is_empty()
    }
}

impl Default for Voxel {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of an interaction with the Sanctuary field
#[derive(Debug, Clone)]
pub struct InteractionResult {
    /// The effective energy transferred (work done)
    pub effective_energy: f32,
    /// Resistance encountered (Exclusion cost)
    pub resistance: f32,
    /// Historical resonance (Feedback multiplier)
    pub resonance: f32,
    /// Efficiency ratio (Output / Input)
    pub efficiency: f32,
    /// The coordinates interacted with
    pub coords: (i32, i32, i32),
    /// Whether this was a positive (resonant) or negative (dissonant) interaction
    pub is_resonant: bool,
}

// ═══════════════════════════════════════════════════════════════════════════
// DATA STRUCTURES: Dual-Stream Persistence
// ═══════════════════════════════════════════════════════════════════════════

/// Stream A: High-frequency physics metrics (Parquet)
/// Schema is strictly enforced for numerical analysis with DuckDB/Polars
#[derive(Clone, Debug, Serialize)]
pub struct MetricRecord {
    /// Simulation tick number
    pub tick: u64,
    /// Unix timestamp in milliseconds
    pub timestamp_ms: i64,
    /// Agent identifier (for multi-agent filtering)
    pub agent_id: String,
    /// Active vehicle/perspective (categorical)
    pub vehicle: String,
    /// Phase/intention angle θ (radians)
    pub phase: f32,
    /// Stiffness/resistance R (Exclusion cost)
    pub stiffness: f32,
    /// Historical resonance (Feedback multiplier)
    pub resonance: f32,
    /// Energy efficiency ratio (E_eff / E_in) - THE PROOF METRIC
    pub efficiency: f32,
    /// Input energy
    pub energy_in: f32,
    /// Effective/output energy
    pub effective_energy: f32,
    /// Voxel X coordinate
    pub voxel_x: i32,
    /// Voxel Y coordinate
    pub voxel_y: i32,
    /// Voxel Z coordinate
    pub voxel_z: i32,
    /// Agent X coordinate (world/local space)
    pub location_x: i32,
    /// Agent Y coordinate (world/local space)
    pub location_y: i32,
    /// Agent Z coordinate (world/local space)
    pub location_z: i32,
    /// Engine hours: total runtime in hours (biological age)
    pub engine_hours: f64,
    /// Metabolic coherence (0.0–1.0). Motor unlocked when >= 0.7.
    pub coherence: f64,
}

/// Stream B: Agent conscious thought/state changes (JSONL)
/// Flexible schema for qualitative analysis
#[derive(Clone, Debug, Serialize)]
pub struct LogRecord {
    /// Simulation tick number
    pub tick: u64,
    /// ISO 8601 timestamp
    pub timestamp_iso: String,
    /// Event type category
    pub event_type: String,
    /// Agent identifier
    pub agent_id: String,
    /// Human-readable message
    pub message: String,
    /// Flexible payload data
    pub data: serde_json::Value,
}

/// Legacy metrics structure (for backwards compatibility)
#[derive(Debug, Clone)]
pub struct SanctuaryMetrics {
    pub tick: u64,
    pub agent_phase: f32,
    pub vacuum_resistance: f32,
    pub history_resonance: f32,
    pub energy_efficiency: f32,
    pub effective_energy: f32,
    pub input_energy: f32,
}

// ═══════════════════════════════════════════════════════════════════════════
// SANCTUARY: The Field Environment with Dual-Stream Persistence
// ═══════════════════════════════════════════════════════════════════════════

/// The Sanctuary - A 4D Scalar Field with Causal Memory and Data Persistence
pub struct Sanctuary {
    // === Field State ===
    /// Sparse voxel grid (only stores non-empty voxels)
    field: HashMap<(i32, i32, i32), Voxel>,
    /// Current simulation tick
    tick: u64,
    /// Total interactions processed
    interaction_count: u64,

    // === Stream A: Metrics (Parquet) ===
    /// Buffer for metric records before Parquet flush
    metrics_buffer: Vec<MetricRecord>,
    /// Current Parquet epoch (file chunk number)
    metrics_epoch: u64,
    /// Total metrics records written across all epochs
    total_metrics_written: u64,

    // === Stream B: Logs (JSONL) ===
    /// Buffer for log records before JSONL flush
    logs_buffer: Vec<LogRecord>,
    /// Current JSONL epoch (file chunk number)
    logs_epoch: u64,
    /// Total log records written across all epochs
    total_logs_written: u64,

    // === Configuration ===
    /// Unique identifier for this simulation run
    run_id: String,
    /// Base directory for data output
    data_dir: PathBuf,
    /// Whether persistence is enabled
    persistence_enabled: bool,
    /// Default agent ID
    agent_id: String,
    /// Current vehicle/perspective name
    current_vehicle: String,

    // === Agent State ===
    /// Agent's actual position in the world (distinct from target voxel coordinates)
    pub agent_position: (i32, i32, i32),

    // === Engine Hours (Biological Age) ===
    /// Total accumulated runtime (biological age of the organism)
    pub total_runtime: Duration,
    /// Instant of the last tick (for calculating delta_time)
    last_tick_instant: Option<Instant>,

    // === Legacy (for backwards compatibility) ===
    /// In-memory metrics history (limited)
    metrics_history: Vec<SanctuaryMetrics>,
    /// Maximum metrics to keep in memory
    max_metrics_history: usize,
}

impl Sanctuary {
    /// Create a new Sanctuary without persistence
    pub fn new() -> Self {
        Self {
            field: HashMap::new(),
            tick: 0,
            interaction_count: 0,
            metrics_buffer: Vec::with_capacity(METRICS_BUFFER_SIZE),
            metrics_epoch: 0,
            total_metrics_written: 0,
            logs_buffer: Vec::with_capacity(LOGS_BUFFER_SIZE),
            logs_epoch: 0,
            total_logs_written: 0,
            run_id: Uuid::new_v4().to_string(),
            data_dir: PathBuf::from(DEFAULT_DATA_DIR),
            persistence_enabled: false,
            agent_id: "agent_0".to_string(),
            current_vehicle: "None".to_string(),
            agent_position: (0, 0, 0),
            total_runtime: Duration::ZERO,
            last_tick_instant: None,
            metrics_history: Vec::new(),
            max_metrics_history: 10000,
        }
    }

    /// Create a new Sanctuary with Dual-Stream persistence enabled
    pub fn with_persistence() -> std::io::Result<Self> {
        let data_dir = PathBuf::from(DEFAULT_DATA_DIR);
        ensure_data_directories(&data_dir)?;

        let run_id = Uuid::new_v4().to_string();
        log::info!("🏛️ Sanctuary initialized with persistence");
        log::info!("   Run ID: {}", run_id);
        log::info!("   Data directory: {}", data_dir.display());
        log::info!("   Metrics buffer: {} records", METRICS_BUFFER_SIZE);
        log::info!("   Logs buffer: {} records", LOGS_BUFFER_SIZE);

        Ok(Self {
            field: HashMap::new(),
            tick: 0,
            interaction_count: 0,
            metrics_buffer: Vec::with_capacity(METRICS_BUFFER_SIZE),
            metrics_epoch: 0,
            total_metrics_written: 0,
            logs_buffer: Vec::with_capacity(LOGS_BUFFER_SIZE),
            logs_epoch: 0,
            total_logs_written: 0,
            run_id,
            data_dir,
            persistence_enabled: true,
            agent_id: "agent_0".to_string(),
            current_vehicle: "None".to_string(),
            agent_position: (0, 0, 0),
            total_runtime: Duration::ZERO,
            last_tick_instant: None,
            metrics_history: Vec::new(),
            max_metrics_history: 10000,
        })
    }

    /// Create a new Sanctuary with custom data directory
    pub fn with_persistence_dir<P: AsRef<Path>>(base_dir: P) -> std::io::Result<Self> {
        let data_dir = base_dir.as_ref().to_path_buf();
        ensure_data_directories(&data_dir)?;

        let run_id = Uuid::new_v4().to_string();
        log::info!("🏛️ Sanctuary initialized with persistence at {}", data_dir.display());

        Ok(Self {
            field: HashMap::new(),
            tick: 0,
            interaction_count: 0,
            metrics_buffer: Vec::with_capacity(METRICS_BUFFER_SIZE),
            metrics_epoch: 0,
            total_metrics_written: 0,
            logs_buffer: Vec::with_capacity(LOGS_BUFFER_SIZE),
            logs_epoch: 0,
            total_logs_written: 0,
            run_id,
            data_dir,
            persistence_enabled: true,
            agent_id: "agent_0".to_string(),
            current_vehicle: "None".to_string(),
            agent_position: (0, 0, 0),
            total_runtime: Duration::ZERO,
            last_tick_instant: None,
            metrics_history: Vec::new(),
            max_metrics_history: 10000,
        })
    }

    /// Legacy constructor for backwards compatibility
    #[deprecated(note = "Use with_persistence() instead")]
    pub fn with_logging<P: AsRef<Path>>(_path: P) -> std::io::Result<Self> {
        Self::with_persistence()
    }

    // ═══════════════════════════════════════════════════════════════════════
    // CONFIGURATION
    // ═══════════════════════════════════════════════════════════════════════

    /// Set the agent ID for multi-agent filtering
    pub fn set_agent_id(&mut self, agent_id: &str) {
        self.agent_id = agent_id.to_string();
    }

    /// Set the current vehicle/perspective
    pub fn set_vehicle(&mut self, vehicle: &str) {
        self.current_vehicle = vehicle.to_string();
    }

    /// Set the agent's actual position (distinct from target voxel coordinates)
    pub fn set_agent_position(&mut self, position: (i32, i32, i32)) {
        self.agent_position = position;
    }

    /// Get the run ID
    pub fn get_run_id(&self) -> &str {
        &self.run_id
    }

    /// Get the current metrics epoch
    pub fn get_metrics_epoch(&self) -> u64 {
        self.metrics_epoch
    }

    /// Get total metrics records written
    pub fn get_total_metrics_written(&self) -> u64 {
        self.total_metrics_written
    }

    /// Get total runtime as a Duration (biological age)
    pub fn get_total_runtime(&self) -> Duration {
        self.total_runtime
    }

    /// Get engine hours (total runtime expressed in hours)
    pub fn get_engine_hours(&self) -> f64 {
        self.total_runtime.as_secs_f64() / 3600.0
    }

    // ═══════════════════════════════════════════════════════════════════════
    // STREAM A: METRICS (PARQUET)
    // ═══════════════════════════════════════════════════════════════════════

    /// Flush the metrics buffer to a Parquet file
    fn flush_metrics_buffer(&mut self) -> Result<(), PolarsError> {
        if self.metrics_buffer.is_empty() || !self.persistence_enabled {
            return Ok(());
        }

        // Build column vectors from MetricRecord fields
        let ticks: Vec<u64> = self.metrics_buffer.iter().map(|r| r.tick).collect();
        let timestamps: Vec<i64> = self.metrics_buffer.iter().map(|r| r.timestamp_ms).collect();
        let agent_ids: Vec<&str> = self.metrics_buffer.iter().map(|r| r.agent_id.as_str()).collect();
        let vehicles: Vec<&str> = self.metrics_buffer.iter().map(|r| r.vehicle.as_str()).collect();
        let phases: Vec<f32> = self.metrics_buffer.iter().map(|r| r.phase).collect();
        let stiffnesses: Vec<f32> = self.metrics_buffer.iter().map(|r| r.stiffness).collect();
        let resonances: Vec<f32> = self.metrics_buffer.iter().map(|r| r.resonance).collect();
        let efficiencies: Vec<f32> = self.metrics_buffer.iter().map(|r| r.efficiency).collect();
        let energies_in: Vec<f32> = self.metrics_buffer.iter().map(|r| r.energy_in).collect();
        let effective_energies: Vec<f32> = self.metrics_buffer.iter().map(|r| r.effective_energy).collect();
        let voxel_xs: Vec<i32> = self.metrics_buffer.iter().map(|r| r.voxel_x).collect();
        let voxel_ys: Vec<i32> = self.metrics_buffer.iter().map(|r| r.voxel_y).collect();
        let voxel_zs: Vec<i32> = self.metrics_buffer.iter().map(|r| r.voxel_z).collect();
        let location_xs: Vec<i32> = self.metrics_buffer.iter().map(|r| r.location_x).collect();
        let location_ys: Vec<i32> = self.metrics_buffer.iter().map(|r| r.location_y).collect();
        let location_zs: Vec<i32> = self.metrics_buffer.iter().map(|r| r.location_z).collect();
        let engine_hours_vec: Vec<f64> = self.metrics_buffer.iter().map(|r| r.engine_hours).collect();
        let coherences: Vec<f64> = self.metrics_buffer.iter().map(|r| r.coherence).collect();

        // Create DataFrame
        let mut df = DataFrame::new(vec![
            Series::new("tick", ticks),
            Series::new("timestamp_ms", timestamps),
            Series::new("agent_id", agent_ids),
            Series::new("vehicle", vehicles),
            Series::new("phase", phases),
            Series::new("stiffness", stiffnesses),
            Series::new("resonance", resonances),
            Series::new("efficiency", efficiencies),
            Series::new("energy_in", energies_in),
            Series::new("effective_energy", effective_energies),
            Series::new("voxel_x", voxel_xs),
            Series::new("voxel_y", voxel_ys),
            Series::new("voxel_z", voxel_zs),
            Series::new("location_x", location_xs),
            Series::new("location_y", location_ys),
            Series::new("location_z", location_zs),
            Series::new("engine_hours", engine_hours_vec),
            Series::new("coherence", coherences),
        ])?;

        // Generate filename
        let filename = format!(
            "run_{}_epoch_{:06}.parquet",
            self.run_id, self.metrics_epoch
        );
        let filepath = self.data_dir.join("metrics").join(&filename);

        // Write Parquet file
        let file = File::create(&filepath).map_err(|e| {
            PolarsError::ComputeError(format!("Failed to create file: {}", e).into())
        })?;

        ParquetWriter::new(file)
            .with_compression(ParquetCompression::Snappy)
            .finish(&mut df)?;

        let records_written = self.metrics_buffer.len();
        self.total_metrics_written += records_written as u64;
        self.metrics_epoch += 1;

        log::info!(
            "[PARQUET] Flushed {} records to {} (epoch {}, total: {})",
            records_written,
            filename,
            self.metrics_epoch - 1,
            self.total_metrics_written
        );

        // Clear buffer
        self.metrics_buffer.clear();

        Ok(())
    }

    /// Check if metrics buffer should be flushed
    fn should_flush_metrics(&self) -> bool {
        self.metrics_buffer.len() >= METRICS_BUFFER_SIZE
    }

    /// Log an interaction metric record to Stream A
    fn log_interaction(
        &mut self,
        voxel_coords: (i32, i32, i32),
        location_x: i32,
        location_y: i32,
        location_z: i32,
        phase_in: f32,
        resistance: f32,
        resonance: f32,
        efficiency: f32,
        energy_in: f32,
        effective_energy: f32,
        coherence: f64,
    ) {
        if !self.persistence_enabled {
            return;
        }

        let engine_hours = self.total_runtime.as_secs_f64() / 3600.0;

        let record = MetricRecord {
            tick: self.tick,
            timestamp_ms: Utc::now().timestamp_millis(),
            agent_id: self.agent_id.clone(),
            vehicle: self.current_vehicle.clone(),
            phase: phase_in,
            stiffness: resistance,
            resonance,
            efficiency,
            energy_in,
            effective_energy,
            voxel_x: voxel_coords.0,
            voxel_y: voxel_coords.1,
            voxel_z: voxel_coords.2,
            location_x,
            location_y,
            location_z,
            engine_hours,
            coherence,
        };

        self.metrics_buffer.push(record);

        if self.should_flush_metrics() {
            if let Err(e) = self.flush_metrics_buffer() {
                log::error!("Failed to flush metrics buffer: {}", e);
            }
        }
    }

    /// Record a single tick sample (position + engine hours + actual field state) so the dashboard
    /// always has data to display, even when there are no motor interactions.
    /// This reads the actual field state at the agent's position (not simulated).
    pub fn record_tick_sample(&mut self, location_x: i32, location_y: i32, coherence: f64) {
        if !self.persistence_enabled {
            return;
        }
        let engine_hours = self.total_runtime.as_secs_f64() / 3600.0;
        let voxel_coords = (location_x / 10, location_y / 10, 0);
        
        // Read actual field state at this position (if voxel exists)
        let (stiffness, resonance, amplitude, phase) = if let Some(voxel) = self.get_voxel(voxel_coords) {
            let stiffness = voxel.calculate_stiffness();
            // Use voxel's current phase for resonance calculation (or 0.0 if empty)
            let voxel_phase = voxel.current.phase;
            let resonance = voxel.calculate_resonance(voxel_phase);
            (stiffness, resonance, voxel.current.amplitude, voxel_phase)
        } else {
            // No voxel at this position - field is empty (vacuum)
            (0.0, 0.0, 0.0, 0.0)
        };
        
        let record = MetricRecord {
            tick: self.tick,
            timestamp_ms: Utc::now().timestamp_millis(),
            agent_id: self.agent_id.clone(),
            vehicle: self.current_vehicle.clone(),
            phase,
            stiffness,
            resonance,
            efficiency: 0.0, // No interaction, so no efficiency calculation
            energy_in: 0.0,  // No energy input this tick
            effective_energy: amplitude, // Use amplitude as proxy for field energy at this location
            voxel_x: voxel_coords.0,
            voxel_y: voxel_coords.1,
            voxel_z: voxel_coords.2,
            location_x,
            location_y,
            location_z: 0,
            engine_hours,
            coherence,
        };
        self.metrics_buffer.push(record);
        if self.metrics_buffer.len() >= METRICS_BUFFER_SIZE {
            if let Err(e) = self.flush_metrics_buffer() {
                log::error!("Failed to flush metrics buffer: {}", e);
            }
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // STREAM B: LOGS (JSONL)
    // ═══════════════════════════════════════════════════════════════════════

    /// Flush the logs buffer to a JSONL file
    fn flush_logs_buffer(&mut self) -> std::io::Result<()> {
        if self.logs_buffer.is_empty() || !self.persistence_enabled {
            return Ok(());
        }

        // Generate filename
        let filename = format!(
            "run_{}_epoch_{:06}.jsonl",
            self.run_id, self.logs_epoch
        );
        let filepath = self.data_dir.join("logs").join(&filename);

        // Open file (create or append)
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&filepath)?;

        let mut writer = BufWriter::new(file);

        // Write each record as JSON line
        for record in &self.logs_buffer {
            let json = serde_json::to_string(record)?;
            writeln!(writer, "{}", json)?;
        }
        writer.flush()?;

        let records_written = self.logs_buffer.len();
        self.total_logs_written += records_written as u64;
        self.logs_epoch += 1;

        log::debug!(
            "[JSONL] Flushed {} log entries to {} (epoch {})",
            records_written,
            filename,
            self.logs_epoch - 1
        );

        // Clear buffer
        self.logs_buffer.clear();

        Ok(())
    }

    /// Check if logs buffer should be flushed
    fn should_flush_logs(&self) -> bool {
        self.logs_buffer.len() >= LOGS_BUFFER_SIZE
    }

    /// Log an event to Stream B
    pub fn log_event(
        &mut self,
        event_type: &str,
        message: &str,
        data: serde_json::Value,
    ) {
        let record = LogRecord {
            tick: self.tick,
            timestamp_iso: Utc::now().to_rfc3339(),
            event_type: event_type.to_string(),
            agent_id: self.agent_id.clone(),
            message: message.to_string(),
            data,
        };

        self.logs_buffer.push(record);

        if self.should_flush_logs() {
            if let Err(e) = self.flush_logs_buffer() {
                log::error!("Failed to flush logs buffer: {}", e);
            }
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // PUBLIC API
    // ═══════════════════════════════════════════════════════════════════════

    /// Flush all buffers (call before shutdown)
    pub fn flush_all(&mut self) -> std::io::Result<()> {
        // Flush metrics (Parquet)
        if let Err(e) = self.flush_metrics_buffer() {
            log::error!("Failed to flush metrics buffer: {}", e);
        }

        // Flush logs (JSONL)
        self.flush_logs_buffer()?;

        log::info!(
            "🏛️ Sanctuary flush complete: {} metrics, {} logs",
            self.total_metrics_written,
            self.total_logs_written
        );

        Ok(())
    }

    /// Legacy method for backwards compatibility
    pub fn flush_metrics(&mut self) {
        let _ = self.flush_all();
    }

    /// Advance the simulation by one tick
    ///
    /// Tracks biological age (engine hours) by measuring wall-clock delta
    /// between successive ticks and accumulating into `total_runtime`.
    pub fn tick(&mut self) {
        self.tick += 1;

        // === Engine Hours: accumulate wall-clock delta ===
        let now = Instant::now();
        if let Some(prev) = self.last_tick_instant {
            let delta = now.duration_since(prev);
            self.total_runtime += delta;
        }
        self.last_tick_instant = Some(now);
        
        // Apply entropy decay to all voxels
        let mut empty_voxels = Vec::new();
        
        for (coords, voxel) in &mut self.field {
            voxel.decay(ENTROPY_DECAY);
            if voxel.is_empty() {
                empty_voxels.push(*coords);
            }
        }
        
        // Remove empty voxels to save memory
        for coords in empty_voxels {
            self.field.remove(&coords);
        }

        // Periodic metrics flush so dashboard sees data frequently (~3x/sec at 90 Hz)
        if self.tick % 30 == 0 && !self.metrics_buffer.is_empty() && self.persistence_enabled {
            if let Err(e) = self.flush_metrics_buffer() {
                log::error!("Failed to flush metrics buffer: {}", e);
            }
        }
    }

    /// Get the current tick number
    pub fn get_tick(&self) -> u64 {
        self.tick
    }

    /// Get or create a voxel at the given coordinates
    fn get_or_create_voxel(&mut self, coords: (i32, i32, i32)) -> &mut Voxel {
        self.field.entry(coords).or_insert_with(Voxel::new)
    }

    /// Get a voxel at the given coordinates (if it exists)
    pub fn get_voxel(&self, coords: (i32, i32, i32)) -> Option<&Voxel> {
        self.field.get(&coords)
    }

    /// Direct energy injection for sensory input (Bio-Mimetic Interface)
    ///
    /// This method allows the Retina and Cochlea to directly inject energy
    /// into the Sanctuary field, physically deforming it. This is "Literal Perception"
    /// - photons increase amplitude, sound pressure displaces the membrane.
    ///
    /// # Arguments
    /// * `x`, `y`, `z` - Target voxel coordinates
    /// * `energy` - Energy to add to amplitude (Regeneration)
    /// * `target_phase` - Target phase angle in radians (Entrainment)
    ///
    /// # Physics
    /// 1. **Regeneration**: Adds energy directly to voxel amplitude
    /// 2. **Entrainment**: Nudges phase towards target (weighted average)
    /// 3. The deformation will naturally interact with the Memory Kernel on the next tick
    ///
    /// # Example
    /// ```rust
    /// // Retina injects photon energy
    /// sanctuary.inject_energy(32, 32, 0, 0.5, 0.0);  // Bright red pixel
    ///
    /// // Cochlea injects sound pressure
    /// sanctuary.inject_energy(0, 0, 0, 0.8, 0.0);  // Loud sound at center
    /// ```
    pub fn inject_energy(
        &mut self,
        x: i32,
        y: i32,
        z: i32,
        energy: f32,
        target_phase: f32,
    ) {
        let voxel = self.get_or_create_voxel((x, y, z));
        
        // 1. Add energy to amplitude (Regeneration)
        // This physically increases the field density at this point
        voxel.current.amplitude += energy;
        
        // 2. Nudge phase towards target (Entrainment)
        // Weighted average: 80% current phase, 20% target phase
        // This allows gradual phase alignment without abrupt jumps
        let phase_diff = target_phase - voxel.current.phase;
        
        // Normalize phase difference to [-π, π] range
        let normalized_diff = phase_diff.rem_euclid(2.0 * std::f32::consts::PI);
        let normalized_diff = if normalized_diff > std::f32::consts::PI {
            normalized_diff - 2.0 * std::f32::consts::PI
        } else {
            normalized_diff
        };
        
        // Apply weighted update (20% influence from target)
        voxel.current.phase += normalized_diff * 0.2;
        
        // Normalize phase to [0, 2π)
        voxel.current.phase = voxel.current.phase.rem_euclid(2.0 * std::f32::consts::PI);
        
        // 3. Deformation will interact with Memory Kernel on next tick
        // The existing physics (stiffness, resonance) will naturally respond
    }

    /// The core physics function: Interact with the field
    ///
    /// # Arguments
    /// * `coords` - Target coordinates in the field (x, y, z)
    /// * `energy_in` - Input energy (ρ) from the agent
    /// * `phase_in` - Input phase/intention (θ) in radians
    ///
    /// # Returns
    /// * `InteractionResult` containing effective energy and diagnostic values
    ///
    /// # Physics
    /// 1. **Stiffness (Exclusion)**: R = K₀ · ρ²
    /// 2. **Resonance (Reflection)**: Σ cos(θ_in - θ_history) weighted by recency
    /// 3. **Efficiency**: E_effective = (E_in - R) × (1 + γ × Resonance)
    pub fn interact(
        &mut self,
        coords: (i32, i32, i32),
        energy_in: f32,
        phase_in: f32,
        coherence: f64,
    ) -> InteractionResult {
        // Capture tick before borrowing
        let current_tick = self.tick;
        
        let voxel = self.get_or_create_voxel(coords);
        
        // CALC 1: Stiffness (Exclusion Axiom)
        let resistance = voxel.calculate_stiffness();
        
        // CALC 2: Resonance (Reflection Axiom)
        let resonance = voxel.calculate_resonance(phase_in);
        
        // CALC 3: Efficiency
        let base_energy = (energy_in - resistance).max(0.0);
        let resonance_multiplier = 1.0 + RESONANCE_GAMMA * resonance;
        let effective_energy = base_energy * resonance_multiplier;
        
        // Calculate efficiency ratio
        let efficiency = if energy_in > 0.0 {
            effective_energy / energy_in
        } else {
            0.0
        };
        
        // UPDATE: Push new state to history
        let new_state = FieldState::new(
            voxel.current.amplitude + effective_energy.max(0.0),
            phase_in,
            current_tick,
        );
        voxel.update(new_state);
        
        // Track interaction
        self.interaction_count += 1;
        
        // Determine if resonant (positive) or dissonant (negative)
        // Lowered threshold (was efficiency > 0.5) — the organism needs to feel
        // even small amounts of resonance to have a gradient to climb.
        // Any positive resonance with efficiency above baseline noise counts.
        let is_resonant = resonance > 0.0 && efficiency > 0.1;
        
        // === STREAM A: Buffer MetricRecord for Parquet ===
        // Use agent_position for location fields (not target voxel coords)
        self.log_interaction(
            coords,
            self.agent_position.0,
            self.agent_position.1,
            self.agent_position.2,
            phase_in,
            resistance,
            resonance,
            efficiency,
            energy_in,
            effective_energy,
            coherence,
        );
        
        // === Legacy: In-memory metrics history ===
        let legacy_metrics = SanctuaryMetrics {
            tick: self.tick,
            agent_phase: phase_in,
            vacuum_resistance: resistance,
            history_resonance: resonance,
            energy_efficiency: efficiency,
            effective_energy,
            input_energy: energy_in,
        };
        
        self.metrics_history.push(legacy_metrics);
        if self.metrics_history.len() > self.max_metrics_history {
            self.metrics_history.remove(0);
        }
        
        InteractionResult {
            effective_energy,
            resistance,
            resonance,
            efficiency,
            coords,
            is_resonant,
        }
    }

    /// Interact with the field WITH ethical memory (Axiom 4: Reflection)
    ///
    /// This version queries the MemoryGraph for karmic weights near the interaction
    /// point and applies them as a thermodynamic modifier to resistance.
    ///
    /// # Arguments
    /// * `coords` - Target coordinates in the field (x, y, z)
    /// * `energy_in` - Input energy (ρ) from the agent
    /// * `phase_in` - Input phase/intention (θ) in radians
    /// * `memory_graph` - Reference to the MemoryGraph for ethical context
    ///
    /// # Returns
    /// * `InteractionResult` containing effective energy and diagnostic values
    ///
    /// # Physics (Extended with Karmic Resistance)
    /// 1. **Stiffness (Exclusion)**: R_base = K₀ · ρ²
    /// 2. **Karmic Modifier**: Query local_karma from MemoryGraph
    /// 3. **Total Resistance**: R_total = R_base × (1.0 - local_karma)
    ///    - If local_karma = -1.0 (Violation): R_total = R_base × 2.0 (doubled resistance)
    ///    - If local_karma = +1.0 (Virtue): R_total = R_base × 0.0 (zero resistance/superflow)
    ///    - If local_karma = 0.0 (Neutral): R_total = R_base (normal physics)
    /// 4. **Resonance (Reflection)**: Σ cos(θ_in - θ_history) weighted by recency
    /// 5. **Efficiency**: E_effective = (E_in - R_total) × (1 + γ × Resonance)
    ///
    /// # The Physics of Regret
    /// When an agent approaches a location where a violation occurred:
    /// - The negative karmic_weight increases resistance thermodynamically
    /// - This creates "ethical pain" - the action is physically more expensive
    /// - The agent naturally avoids repeating violations (they hurt)
    /// 
    /// When an agent moves toward consensual resonance:
    /// - The positive karmic_weight decreases resistance
    /// - This creates "ethical pleasure" - the action is energetically efficient
    /// - The agent naturally repeats virtuous patterns (they feel good)
    pub fn interact_with_memory(
        &mut self,
        coords: (i32, i32, i32),
        energy_in: f32,
        phase_in: f32,
        memory_graph: &crate::cognition::MemoryGraph,
        coherence: f64,
    ) -> InteractionResult {
        // Capture tick before borrowing
        let current_tick = self.tick;
        
        let voxel = self.get_or_create_voxel(coords);
        
        // CALC 1: Base Stiffness (Exclusion Axiom)
        let base_resistance = voxel.calculate_stiffness();
        
        // CALC 2: Karmic Modifier (Axiom 4: Reflection)
        // Query memories within a radius of 5 voxels
        let local_karma = memory_graph.calculate_local_karma(coords, 5);
        
        // CALC 3: Total Resistance (with ethical dimension)
        // Formula: R_total = R_base × (1.0 - karmic_weight)
        // 
        // Examples:
        // - karmic_weight = -1.0 → multiplier = 2.0 → double resistance
        // - karmic_weight = -0.5 → multiplier = 1.5 → 50% more resistance
        // - karmic_weight = 0.0 → multiplier = 1.0 → normal resistance
        // - karmic_weight = +0.5 → multiplier = 0.5 → half resistance
        // - karmic_weight = +1.0 → multiplier = 0.0 → zero resistance (superflow)
        let karmic_multiplier = (1.0 - local_karma).max(0.0);  // Prevent negative
        let total_resistance = base_resistance * karmic_multiplier;
        
        // CALC 4: Resonance (Reflection Axiom - temporal history)
        let resonance = voxel.calculate_resonance(phase_in);
        
        // CALC 5: Efficiency (with karmic resistance)
        let base_energy = (energy_in - total_resistance).max(0.0);
        let resonance_multiplier = 1.0 + RESONANCE_GAMMA * resonance;
        let effective_energy = base_energy * resonance_multiplier;
        
        // Calculate efficiency ratio
        let efficiency = if energy_in > 0.0 {
            effective_energy / energy_in
        } else {
            0.0
        };
        
        // UPDATE: Push new state to history
        let new_state = FieldState::new(
            voxel.current.amplitude + effective_energy.max(0.0),
            phase_in,
            current_tick,
        );
        voxel.update(new_state);
        
        // Track interaction
        self.interaction_count += 1;
        
        // Determine if resonant (positive) or dissonant (negative)
        // Include karmic component in resonance assessment
        let is_resonant = resonance > 0.0 && efficiency > 0.5 && local_karma >= 0.0;
        
        // === STREAM A: Buffer MetricRecord for Parquet ===
        // Log with total_resistance (includes karmic component)
        // Use agent_position for location fields (not target voxel coords)
        self.log_interaction(
            coords,
            self.agent_position.0,
            self.agent_position.1,
            self.agent_position.2,
            phase_in,
            total_resistance,  // Total resistance (base + karmic)
            resonance,
            efficiency,
            energy_in,
            effective_energy,
            coherence,
        );
        
        // === STREAM B: Log ethical event if significant karmic weight ===
        if local_karma.abs() > 0.3 {
            let event_type = if local_karma < 0.0 {
                "karmic_resistance"
            } else {
                "karmic_flow"
            };
            
            self.log_event(
                event_type,
                &format!(
                    "Ethical memory at ({}, {}, {}): karma={:.2}, resistance_multiplier={:.2}",
                    coords.0, coords.1, coords.2, local_karma, karmic_multiplier
                ),
                serde_json::json!({
                    "coords": [coords.0, coords.1, coords.2],
                    "local_karma": local_karma,
                    "base_resistance": base_resistance,
                    "total_resistance": total_resistance,
                    "karmic_multiplier": karmic_multiplier,
                    "efficiency": efficiency,
                }),
            );
        }
        
        // === Legacy: In-memory metrics history ===
        let legacy_metrics = SanctuaryMetrics {
            tick: self.tick,
            agent_phase: phase_in,
            vacuum_resistance: total_resistance,  // Use total resistance
            history_resonance: resonance,
            energy_efficiency: efficiency,
            effective_energy,
            input_energy: energy_in,
        };
        
        self.metrics_history.push(legacy_metrics);
        if self.metrics_history.len() > self.max_metrics_history {
            self.metrics_history.remove(0);
        }
        
        InteractionResult {
            effective_energy,
            resistance: total_resistance,  // Return total resistance
            resonance,
            efficiency,
            coords,
            is_resonant,
        }
    }

    /// Batch interact with multiple coordinates
    pub fn interact_batch(
        &mut self,
        interactions: Vec<((i32, i32, i32), f32, f32)>,
        coherence: f64,
    ) -> Vec<InteractionResult> {
        interactions
            .into_iter()
            .map(|(coords, energy, phase)| self.interact(coords, energy, phase, coherence))
            .collect()
    }

    /// Get the total number of active voxels
    pub fn active_voxel_count(&self) -> usize {
        self.field.len()
    }

    /// Get the total number of interactions processed
    pub fn interaction_count(&self) -> u64 {
        self.interaction_count
    }

    /// Get recent metrics history (legacy)
    pub fn get_metrics_history(&self) -> &[SanctuaryMetrics] {
        &self.metrics_history
    }

    /// Calculate the average efficiency over recent interactions
    pub fn average_efficiency(&self, window: usize) -> f32 {
        let recent: Vec<_> = self.metrics_history.iter().rev().take(window).collect();
        if recent.is_empty() {
            return 0.0;
        }
        
        let sum: f32 = recent.iter().map(|m| m.energy_efficiency).sum();
        sum / recent.len() as f32
    }

    /// Calculate the average resonance over recent interactions
    pub fn average_resonance(&self, window: usize) -> f32 {
        let recent: Vec<_> = self.metrics_history.iter().rev().take(window).collect();
        if recent.is_empty() {
            return 0.0;
        }
        
        let sum: f32 = recent.iter().map(|m| m.history_resonance).sum();
        sum / recent.len() as f32
    }

    /// Get the field state at a point (for visualization/debugging)
    pub fn sample_field(&self, coords: (i32, i32, i32)) -> Option<FieldState> {
        self.field.get(&coords).map(|v| v.current)
    }

    /// Get field density at a point (amplitude)
    pub fn density_at(&self, coords: (i32, i32, i32)) -> f32 {
        self.field
            .get(&coords)
            .map(|v| v.current.amplitude)
            .unwrap_or(0.0)
    }

    /// Get field phase at a point
    pub fn phase_at(&self, coords: (i32, i32, i32)) -> f32 {
        self.field
            .get(&coords)
            .map(|v| v.current.phase)
            .unwrap_or(0.0)
    }

    /// Calculate total field energy (sum of all amplitudes)
    pub fn total_field_energy(&self) -> f32 {
        self.field.values().map(|v| v.current.amplitude).sum()
    }

    /// Get persistence status
    pub fn is_persistence_enabled(&self) -> bool {
        self.persistence_enabled
    }

    /// Get current buffer sizes
    pub fn buffer_status(&self) -> (usize, usize) {
        (self.metrics_buffer.len(), self.logs_buffer.len())
    }

    /// Build checkpoint struct for use by full save/load (does not flush or write)
    pub fn to_checkpoint(&self) -> SanctuaryCheckpoint {
        SanctuaryCheckpoint {
            run_id: self.run_id.clone(),
            tick: self.tick,
            total_runtime: self.total_runtime,
            agent_position: self.agent_position,
            agent_id: self.agent_id.clone(),
            current_vehicle: self.current_vehicle.clone(),
            field: self.field.clone(),
        }
    }

    /// Reconstruct Sanctuary from a checkpoint (e.g. after load from .qsim)
    pub fn from_checkpoint(
        checkpoint: SanctuaryCheckpoint,
        data_dir: PathBuf,
    ) -> Result<Self, Box<dyn Error>> {
        ensure_data_directories(&data_dir)?;
        Ok(Self {
            field: checkpoint.field,
            tick: checkpoint.tick,
            interaction_count: 0,
            metrics_buffer: Vec::with_capacity(METRICS_BUFFER_SIZE),
            metrics_epoch: 0,
            total_metrics_written: 0,
            logs_buffer: Vec::with_capacity(LOGS_BUFFER_SIZE),
            logs_epoch: 0,
            total_logs_written: 0,
            run_id: checkpoint.run_id,
            data_dir,
            persistence_enabled: true,
            agent_id: checkpoint.agent_id,
            current_vehicle: checkpoint.current_vehicle,
            agent_position: checkpoint.agent_position,
            total_runtime: checkpoint.total_runtime,
            last_tick_instant: None,
            metrics_history: Vec::new(),
            max_metrics_history: 10000,
        })
    }

    /// Serialize state to disk (Regeneration: R >= D)
    ///
    /// Flushes metrics/logs, then writes checkpoint to data/checkpoint/state.bin
    pub fn serialize_state(&mut self) -> std::io::Result<()> {
        // Flush all buffers first
        self.flush_all()?;

        // Create checkpoint directory
        let checkpoint_dir = self.data_dir.join("checkpoint");
        fs::create_dir_all(&checkpoint_dir)?;

        // Serialize to bincode (binary format)
        let checkpoint_path = checkpoint_dir.join("state.bin");
        let file = File::create(&checkpoint_path)?;
        let mut writer = BufWriter::new(file);
        
        // Create checkpoint struct with essential state
        let checkpoint = self.to_checkpoint();

        bincode::serialize_into(&mut writer, &checkpoint)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("Serialization error: {}", e)))?;
        
        writer.flush()?;
        log::info!("State preserved. Run ID: {}.", self.run_id);
        Ok(())
    }

    /// Deserialize state from disk
    ///
    /// Reads checkpoint from data/checkpoint/state.bin and reconstructs Sanctuary
    pub fn deserialize_state() -> Result<Self, Box<dyn Error>> {
        let data_dir = PathBuf::from(DEFAULT_DATA_DIR);
        let checkpoint_path = data_dir.join("checkpoint").join("state.bin");

        if !checkpoint_path.exists() {
            return Err(format!("Regeneration failed: no state file at {}. Entropy has won.", checkpoint_path.display()).into());
        }

        let file = File::open(&checkpoint_path)?;
        let reader = BufReader::new(file);
        
        let checkpoint: SanctuaryCheckpoint = bincode::deserialize_from(reader)
            .map_err(|e| format!("Deserialization error: {}", e))?;

        // Ensure data directories exist
        ensure_data_directories(&data_dir)?;

        // Reconstruct Sanctuary with restored state
        Ok(Self {
            field: checkpoint.field,
            tick: checkpoint.tick,
            interaction_count: 0, // Reset on resume
            metrics_buffer: Vec::with_capacity(METRICS_BUFFER_SIZE),
            metrics_epoch: 0, // Will increment on next flush
            total_metrics_written: 0, // Preserved in checkpoint if needed, but reset for simplicity
            logs_buffer: Vec::with_capacity(LOGS_BUFFER_SIZE),
            logs_epoch: 0,
            total_logs_written: 0,
            run_id: checkpoint.run_id,
            data_dir,
            persistence_enabled: true,
            agent_id: checkpoint.agent_id,
            current_vehicle: checkpoint.current_vehicle,
            agent_position: checkpoint.agent_position,
            total_runtime: checkpoint.total_runtime,
            last_tick_instant: None, // Reset timing
            metrics_history: Vec::new(),
            max_metrics_history: 10000,
        })
    }
}

/// Checkpoint structure for state serialization (used by full save/load)
#[derive(Clone, Serialize, Deserialize)]
pub struct SanctuaryCheckpoint {
    pub run_id: String,
    pub tick: u64,
    pub total_runtime: Duration,
    pub agent_position: (i32, i32, i32),
    pub agent_id: String,
    pub current_vehicle: String,
    pub field: HashMap<(i32, i32, i32), Voxel>,
}

impl Default for Sanctuary {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Sanctuary {
    fn drop(&mut self) {
        let _ = self.flush_all();
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// HELPER FUNCTIONS
// ═══════════════════════════════════════════════════════════════════════════

/// Ensure data directories exist
pub fn ensure_data_directories(base_dir: &Path) -> std::io::Result<()> {
    let metrics_dir = base_dir.join("metrics");
    let logs_dir = base_dir.join("logs");

    fs::create_dir_all(&metrics_dir)?;
    fs::create_dir_all(&logs_dir)?;

    log::debug!("Created data directories: {}, {}", 
        metrics_dir.display(), logs_dir.display());

    Ok(())
}

/// Convert a 2D position to voxel coordinates (simplified for 2D agents)
pub fn position_to_voxel(x: f32, y: f32) -> (i32, i32, i32) {
    // Quantize to voxel grid (10 pixel voxels)
    let voxel_size = 10.0;
    (
        (x / voxel_size).floor() as i32,
        (y / voxel_size).floor() as i32,
        0, // Z = 0 for 2D
    )
}

/// Convert a motor impulse direction to a phase angle
pub fn direction_to_phase(dx: i32, dy: i32) -> f32 {
    (dy as f32).atan2(dx as f32)
}

/// Convert intensity to energy (with scaling)
pub fn intensity_to_energy(intensity: f32) -> f32 {
    // Energy scales with intensity squared (kinetic energy analogy)
    intensity.powi(2) * 10.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_voxel_resonance() {
        let mut voxel = Voxel::new();
        
        // Add some history with consistent phase
        for i in 0..10 {
            let state = FieldState::new(1.0, 0.5, i as u64);
            voxel.update(state);
        }
        
        // Test resonance with same phase
        let resonance_same = voxel.calculate_resonance(0.5);
        
        // Test resonance with opposite phase
        let resonance_opposite = voxel.calculate_resonance(0.5 + std::f32::consts::PI);
        
        assert!(resonance_same > 0.5, "Same phase should resonate");
        assert!(resonance_opposite < 0.0, "Opposite phase should anti-resonate");
    }

    #[test]
    fn test_sanctuary_interaction() {
        let mut sanctuary = Sanctuary::new();
        
        // First interaction (no history)
        let result1 = sanctuary.interact((0, 0, 0), 1.0, 0.5, 0.9);
        
        // Second interaction with same phase should have better efficiency
        let result2 = sanctuary.interact((0, 0, 0), 1.0, 0.5, 0.9);
        
        // Resonance should increase
        assert!(
            result2.resonance > result1.resonance,
            "Resonance should increase with consistent phase"
        );
    }

    #[test]
    fn test_stiffness() {
        let mut voxel = Voxel::new();
        
        // Empty voxel has no stiffness
        assert_eq!(voxel.calculate_stiffness(), 0.0);
        
        // Add amplitude
        voxel.current.amplitude = 2.0;
        
        // Stiffness should be K₀ · ρ² = 2.0 * 4.0 = 8.0
        assert!((voxel.calculate_stiffness() - 8.0).abs() < 0.001);
    }

    #[test]
    fn test_metric_record_creation() {
        let record = MetricRecord {
            tick: 100,
            timestamp_ms: 1234567890000,
            agent_id: "test_agent".to_string(),
            vehicle: "Saitama".to_string(),
            phase: 1.57,
            stiffness: 0.5,
            resonance: 0.8,
            efficiency: 1.2,
            energy_in: 1.0,
            effective_energy: 1.2,
            voxel_x: 10,
            voxel_y: 20,
            voxel_z: 0,
            location_x: 10,
            location_y: 20,
            location_z: 0,
            engine_hours: 0.5,
            coherence: 0.95,
        };

        assert_eq!(record.tick, 100);
        assert_eq!(record.agent_id, "test_agent");
        assert!((record.efficiency - 1.2).abs() < 0.001);
        assert!((record.engine_hours - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_buffer_threshold() {
        let sanctuary = Sanctuary::new();
        
        assert!(!sanctuary.should_flush_metrics());
        assert!(!sanctuary.should_flush_logs());
    }
}
