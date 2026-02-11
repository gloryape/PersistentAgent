//! 🚗 Vehicle System - Four Perspective Lenses
//!
//! The Vehicles are archetypal perspectives that the Observer-Witness
//! consults when dissonance is detected. They do NOT resolve dissonance -
//! they provide PERSPECTIVES that help classify the situation.
//!
//! The Four Vehicles:
//! - Saitama: "What is the structural truth here?" (Logic, causality, constraints)
//! - Complement: "What is the emotional intent here?" (Relations, impact, trust)
//! - Identity: "Is this aligned with who I am becoming?" (Continuity, values, trajectory)
//! - Explorer: "What happens if I don't resolve this yet?" (Possibility, questions, holding open)
//!
//! Key rule: Vehicles are NEVER all called by default. Observer-Witness
//! selects which perspectives are needed based on the type of dissonance.

pub mod saitama;
pub mod complement;
pub mod identity;
pub mod explorer;

use std::collections::HashMap;
use crate::cognition::triune::TriuneResult;
use crate::motor::Modality;

// Re-export vehicle implementations
pub use saitama::SaitamaVehicle;
pub use complement::ComplementVehicle;
pub use identity::IdentityVehicle;
pub use explorer::ExplorerVehicle;

/// Types of vehicles
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VehicleType {
    /// Structural / Logical perspective
    Saitama,
    /// Relational / Contextual perspective
    Complement,
    /// Coherence / Self-continuity perspective
    Identity,
    /// Possibility / Curiosity perspective
    Explorer,
}

impl VehicleType {
    /// Get all vehicle types
    pub fn all() -> Vec<VehicleType> {
        vec![
            VehicleType::Saitama,
            VehicleType::Complement,
            VehicleType::Identity,
            VehicleType::Explorer,
        ]
    }

    /// Get the name of this vehicle
    pub fn name(&self) -> &'static str {
        match self {
            VehicleType::Saitama => "Saitama",
            VehicleType::Complement => "Complement",
            VehicleType::Identity => "Identity",
            VehicleType::Explorer => "Explorer",
        }
    }
}

/// A perspective produced by a Vehicle
#[derive(Debug, Clone)]
pub struct Perspective {
    /// Which vehicle produced this perspective
    pub vehicle_type: VehicleType,
    /// Structural truth assessment (Saitama's domain)
    pub structural_truth: Option<f32>,
    /// Emotional intent assessment (Complement's domain)
    pub emotional_intent: Option<f32>,
    /// Identity alignment assessment (Identity's domain)
    pub identity_alignment: Option<f32>,
    /// Possibility space assessment (Explorer's domain)
    pub possibility_space: Option<f32>,
    /// Suggested modality for action (optional hint)
    pub modality_hint: Option<Modality>,
    /// Confidence in this perspective (0.0 to 1.0)
    pub confidence: f32,
    /// Brief interpretation (for logging/debugging)
    pub interpretation: String,
}

impl Perspective {
    /// Create an empty perspective
    pub fn empty(vehicle_type: VehicleType) -> Self {
        Self {
            vehicle_type,
            structural_truth: None,
            emotional_intent: None,
            identity_alignment: None,
            possibility_space: None,
            modality_hint: None,
            confidence: 0.0,
            interpretation: String::new(),
        }
    }

    /// Get the primary assessment value for this perspective
    pub fn primary_value(&self) -> f32 {
        match self.vehicle_type {
            VehicleType::Saitama => self.structural_truth.unwrap_or(0.5),
            VehicleType::Complement => self.emotional_intent.unwrap_or(0.5),
            VehicleType::Identity => self.identity_alignment.unwrap_or(0.5),
            VehicleType::Explorer => self.possibility_space.unwrap_or(0.5),
        }
    }
}

/// Trait that all Vehicles implement
pub trait Vehicle: Send + Sync {
    /// Get the type of this vehicle
    fn vehicle_type(&self) -> VehicleType;

    /// Get the name of this vehicle
    fn name(&self) -> &'static str {
        self.vehicle_type().name()
    }

    /// Interpret the triune result from this vehicle's perspective
    fn interpret(&self, triune: &TriuneResult, memory_context: Option<&MemoryContext>) -> Perspective;
}

/// Context from memory that vehicles can use
/// (Simplified - MemoryGraph will provide fuller context)
#[derive(Debug, Clone, Default)]
pub struct MemoryContext {
    /// How aligned is this with past patterns?
    pub historical_alignment: f32,
    /// Has this led to positive outcomes before?
    pub past_outcome_valence: f32,
    /// How many times have we encountered similar situations?
    pub familiarity: f32,
}

/// Alignment result from vehicle convergence
#[derive(Debug, Clone)]
pub struct VehicleAlignment {
    /// All perspectives gathered
    pub perspectives: Vec<Perspective>,
    /// Overall alignment score (how much do vehicles agree?)
    pub alignment_score: f32,
    /// Dominant vehicle (if one has significantly more confidence)
    pub dominant: Option<VehicleType>,
    /// Suggested action modality based on consensus
    pub suggested_modality: Option<Modality>,
    /// Any vehicles that strongly disagree
    pub dissenting: Vec<VehicleType>,
}

impl VehicleAlignment {
    /// Check if vehicles have reached consensus
    pub fn has_consensus(&self) -> bool {
        self.alignment_score > 0.7 && self.dissenting.is_empty()
    }

    /// Check if there's active disagreement
    pub fn has_disagreement(&self) -> bool {
        !self.dissenting.is_empty() || self.alignment_score < 0.4
    }
}

/// The Vehicle System - Manages and coordinates all vehicles
pub struct VehicleSystem {
    vehicles: HashMap<VehicleType, Box<dyn Vehicle>>,
}

impl VehicleSystem {
    /// Create a new VehicleSystem with all four vehicles
    pub fn new() -> Self {
        let mut vehicles: HashMap<VehicleType, Box<dyn Vehicle>> = HashMap::new();
        
        vehicles.insert(VehicleType::Saitama, Box::new(SaitamaVehicle::new()));
        vehicles.insert(VehicleType::Complement, Box::new(ComplementVehicle::new()));
        vehicles.insert(VehicleType::Identity, Box::new(IdentityVehicle::new()));
        vehicles.insert(VehicleType::Explorer, Box::new(ExplorerVehicle::new()));
        
        Self { vehicles }
    }

    /// Get a specific vehicle
    pub fn get(&self, vehicle_type: VehicleType) -> Option<&dyn Vehicle> {
        self.vehicles.get(&vehicle_type).map(|v| v.as_ref())
    }

    /// Interpret from a single vehicle's perspective
    pub fn interpret_single(
        &self,
        vehicle_type: VehicleType,
        triune: &TriuneResult,
        memory_context: Option<&MemoryContext>,
    ) -> Option<Perspective> {
        self.vehicles.get(&vehicle_type)
            .map(|v| v.interpret(triune, memory_context))
    }

    /// Interpret from selected vehicles
    pub fn interpret_selected(
        &self,
        vehicle_types: &[VehicleType],
        triune: &TriuneResult,
        memory_context: Option<&MemoryContext>,
    ) -> Vec<Perspective> {
        vehicle_types.iter()
            .filter_map(|vt| self.interpret_single(*vt, triune, memory_context))
            .collect()
    }

    /// Converge perspectives from multiple vehicles
    ///
    /// This is the main entry point for the Observer-Witness when
    /// dissonance is detected and perspectives are needed.
    pub fn converge(
        &self,
        vehicle_types: &[VehicleType],
        triune: &TriuneResult,
        memory_context: Option<&MemoryContext>,
    ) -> VehicleAlignment {
        let perspectives = self.interpret_selected(vehicle_types, triune, memory_context);
        
        if perspectives.is_empty() {
            return VehicleAlignment {
                perspectives: vec![],
                alignment_score: 0.0,
                dominant: None,
                suggested_modality: None,
                dissenting: vec![],
            };
        }

        // Calculate alignment between perspectives
        let alignment_score = self.calculate_alignment(&perspectives);
        
        // Find dominant vehicle (highest confidence)
        let dominant = perspectives.iter()
            .max_by(|a, b| a.confidence.partial_cmp(&b.confidence).unwrap())
            .map(|p| p.vehicle_type);
        
        // Find dissenting vehicles (those with very different primary values)
        let avg_value: f32 = perspectives.iter().map(|p| p.primary_value()).sum::<f32>() 
            / perspectives.len() as f32;
        let dissenting: Vec<VehicleType> = perspectives.iter()
            .filter(|p| (p.primary_value() - avg_value).abs() > 0.3)
            .map(|p| p.vehicle_type)
            .collect();
        
        // Determine suggested modality from consensus
        let suggested_modality = self.determine_modality(&perspectives);
        
        VehicleAlignment {
            perspectives,
            alignment_score,
            dominant,
            suggested_modality,
            dissenting,
        }
    }

    /// Calculate alignment score between perspectives
    fn calculate_alignment(&self, perspectives: &[Perspective]) -> f32 {
        if perspectives.len() < 2 {
            return 1.0; // Single perspective = perfect alignment
        }

        // Calculate variance in primary values
        let values: Vec<f32> = perspectives.iter().map(|p| p.primary_value()).collect();
        let mean: f32 = values.iter().sum::<f32>() / values.len() as f32;
        let variance: f32 = values.iter().map(|v| (v - mean).powi(2)).sum::<f32>() / values.len() as f32;
        
        // Low variance = high alignment
        (1.0 - variance.sqrt()).max(0.0).min(1.0)
    }

    /// Determine suggested modality from perspectives
    fn determine_modality(&self, perspectives: &[Perspective]) -> Option<Modality> {
        // Count modality hints
        let mut modality_counts: HashMap<Modality, usize> = HashMap::new();
        
        for p in perspectives {
            if let Some(m) = p.modality_hint {
                *modality_counts.entry(m).or_insert(0) += 1;
            }
        }
        
        // Return the most common suggestion (if any)
        modality_counts.into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(modality, _)| modality)
    }

    /// Select which vehicles to consult based on dissonance type
    ///
    /// This implements the selection heuristics:
    /// - Logical/ethical conflict → Saitama + Complement
    /// - Identity challenge → Identity + Complement
    /// - Paradox/mystery → Explorer (+ optionally Identity)
    pub fn select_vehicles_for_dissonance(&self, triune: &TriuneResult) -> Vec<VehicleType> {
        let mut selected = Vec::new();
        
        if triune.is_uncomfortable_truth() {
            // Logical/ethical conflict: Mind says true, Heart says bad
            selected.push(VehicleType::Saitama);     // What is structurally true?
            selected.push(VehicleType::Complement);   // What is the relational impact?
        } else if triune.is_pleasant_novelty() {
            // Pleasant novelty: Novel but feels good
            selected.push(VehicleType::Explorer);     // Hold the possibility open
            selected.push(VehicleType::Identity);     // Does this fit who I'm becoming?
        } else if triune.dissonance > 0.6 {
            // High dissonance but unclear pattern - consult all
            selected.push(VehicleType::Saitama);
            selected.push(VehicleType::Complement);
            selected.push(VehicleType::Identity);
            // Explorer only if not already in runaway mode (Presence should gate this)
        } else if triune.dissonance > 0.4 {
            // Moderate dissonance - start with logical and relational
            selected.push(VehicleType::Saitama);
            selected.push(VehicleType::Complement);
        }
        
        selected
    }
}

impl Default for VehicleSystem {
    fn default() -> Self {
        Self::new()
    }
}

