//! 🔮 MemoryGraph - Memory As Being
//!
//! The MemoryGraph is NOT a simple storage system. It is the organism's
//! identity manifested as persistent bindings that exert force on interpretation.
//!
//! Key differences from MemoryBank:
//! - MemoryBank: Sensory-level correlations (visual ↔ audio)
//! - MemoryGraph: Meaningful bindings (situations ↔ outcomes ↔ resonances)
//!
//! Write Rules:
//! - Write ONLY on Coherent or DeepMystery outcomes (not NeedsPerspective)
//! - Updates must come through Observer-Witness
//! - Weak links decay unless reinforced
//!
//! This IS identity. Identity Vehicle reads it, never owns it.

use uuid::Uuid;
use std::collections::HashMap;
use std::time::{Instant, Duration};

use crate::motor::MotorImpulse;
use crate::cognition::attention_field::{StimulusContext, MemoryEcho};
use crate::cognition::vehicles::MemoryContext;

/// How long before links start decaying (10 minutes)
const DECAY_GRACE_PERIOD: Duration = Duration::from_secs(600);
/// Minimum strength before a link is removed
const MIN_LINK_STRENGTH: f32 = 0.1;
/// Decay rate per tick (very slow)
const DECAY_RATE: f32 = 0.001;

// ═══════════════════════════════════════════════════════════════════════════
// CONSENT & ETHICAL STRUCTURES - Axiom 4: Reflection
// ═══════════════════════════════════════════════════════════════════════════

/// Result of a consent check for an action or experience
/// This determines the ethical weight (karmic_weight) that gets embedded in memory
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConsentResult {
    /// Action was allowed with positive ethical outcome
    /// The efficiency_score represents how well the action resonated (0.0 to 1.0)
    Allowed { efficiency_score: f32 },
    
    /// Action was denied with reason
    /// Creates negative karmic weight (ethical resistance)
    Denied,
    
    /// Active consent violation occurred
    /// Creates maximum negative karmic weight (ethical wall)
    Violation,
    
    /// Neutral/unknown consent state
    /// No ethical weight applied
    Neutral,
}

/// A signature that captures the essence of a situation
#[derive(Debug, Clone)]
pub struct MemorySignature {
    /// Visual characteristics (simplified)
    pub visual_features: Vec<f32>,
    /// Audio characteristics (simplified)
    pub audio_features: Vec<f32>,
    /// Contextual features (relations, timing)
    pub contextual_features: Vec<f32>,
    /// Triune state at time of encoding
    pub triune_coherence: f32,
    pub triune_dissonance: f32,
    pub experiential_valence: f32,
}

impl MemorySignature {
    /// Create an empty signature
    pub fn empty() -> Self {
        Self {
            visual_features: vec![],
            audio_features: vec![],
            contextual_features: vec![],
            triune_coherence: 0.5,
            triune_dissonance: 0.0,
            experiential_valence: 0.0,
        }
    }

    /// Create from stimulus context
    pub fn from_context(context: &StimulusContext, coherence: f32, dissonance: f32, valence: f32) -> Self {
        // Extract basic features from the stimulus
        let visual_features = match &context.raw.source {
            crate::cognition::stimuli::StimulusSource::VisualRegion { rect, entropy, average_color } => {
                vec![
                    rect.x as f32 / 1920.0,
                    rect.y as f32 / 1080.0,
                    rect.width as f32 / 1920.0,
                    rect.height as f32 / 1080.0,
                    *entropy as f32,
                    average_color.0[0] as f32 / 255.0,
                    average_color.0[1] as f32 / 255.0,
                    average_color.0[2] as f32 / 255.0,
                ]
            }
            _ => vec![],
        };

        let audio_features = match &context.raw.source {
            crate::cognition::stimuli::StimulusSource::AudioStream { frequency, pattern, volume } => {
                vec![
                    *frequency as f32 / 20000.0,
                    *volume as f32,
                    pattern.dominant_freq as f32 / 20000.0,
                    pattern.harmonic_ratio as f32,
                ]
            }
            _ => vec![],
        };

        let contextual_features = vec![
            context.contextual_salience,
            context.relations.len() as f32 / 10.0,
            context.memory_echoes.len() as f32 / 5.0,
        ];

        Self {
            visual_features,
            audio_features,
            contextual_features,
            triune_coherence: coherence,
            triune_dissonance: dissonance,
            experiential_valence: valence,
        }
    }

    /// Calculate similarity to another signature
    pub fn similarity(&self, other: &MemorySignature) -> f32 {
        let visual_sim = Self::vector_similarity(&self.visual_features, &other.visual_features);
        let audio_sim = Self::vector_similarity(&self.audio_features, &other.audio_features);
        let context_sim = Self::vector_similarity(&self.contextual_features, &other.contextual_features);
        
        // Weight visual higher if both have visual, audio higher if both have audio
        let has_visual = !self.visual_features.is_empty() && !other.visual_features.is_empty();
        let has_audio = !self.audio_features.is_empty() && !other.audio_features.is_empty();
        
        if has_visual && has_audio {
            visual_sim * 0.4 + audio_sim * 0.4 + context_sim * 0.2
        } else if has_visual {
            visual_sim * 0.7 + context_sim * 0.3
        } else if has_audio {
            audio_sim * 0.7 + context_sim * 0.3
        } else {
            context_sim
        }
    }

    fn vector_similarity(a: &[f32], b: &[f32]) -> f32 {
        if a.is_empty() || b.is_empty() || a.len() != b.len() {
            return 0.0;
        }
        
        let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
        
        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }
        
        (dot / (norm_a * norm_b)).max(0.0).min(1.0)
    }
}

/// A node in the memory graph - a crystallized experience
#[derive(Debug, Clone)]
pub struct MemoryNode {
    /// Unique identifier
    pub id: Uuid,
    /// The signature of this memory
    pub signature: MemorySignature,
    /// When this memory was first formed
    pub first_observed: Instant,
    /// When this memory was last reinforced
    pub last_reinforced: Instant,
    /// How many times this memory has been reinforced
    pub reinforcement_count: u32,
    /// The outcome that resulted from this situation
    pub outcome_valence: f32,  // -1.0 (bad) to 1.0 (good)
    /// The action that was taken (if any)
    pub outcome_action: Option<MotorImpulse>,
    /// Whether this was classified as DeepMystery
    pub is_mystery: bool,
    
    // === AXIOM 4: REFLECTION - Memory as Being ===
    /// The "Karmic" Weight - Ethical dimension of this memory
    /// Range: -1.0 (Consent Violation) to +1.0 (Consensual Resonance)
    /// Default: 0.0 (Neutral/Unknown)
    /// 
    /// **Physics**: This weight acts as a thermodynamic modifier in Sanctuary.
    /// - Negative values increase resistance (ethical pain)
    /// - Positive values decrease resistance (ethical flow)
    pub karmic_weight: f32,
    
    /// Spatial location in Sanctuary where this memory was formed
    /// Used for locality-based ethical resistance calculations
    pub location: (i32, i32, i32),
}

/// A resonance link between two memory nodes
#[derive(Debug, Clone)]
pub struct ResonanceLink {
    /// Source node ID
    pub from: Uuid,
    /// Target node ID
    pub to: Uuid,
    /// Strength of the link (0.0 to 1.0)
    pub strength: f32,
    /// When this link was last reinforced
    pub last_reinforced: Instant,
    /// Type of resonance
    pub resonance_type: ResonanceType,
}

/// Types of resonance between memories
#[derive(Debug, Clone, PartialEq)]
pub enum ResonanceType {
    /// Similar situations
    Situational,
    /// Similar outcomes
    Outcome,
    /// Causal sequence (A led to B)
    Causal,
    /// Temporal co-occurrence
    Temporal,
}

/// The Memory Graph - Identity manifest as persistent bindings
pub struct MemoryGraph {
    /// All memory nodes
    nodes: HashMap<Uuid, MemoryNode>,
    /// All resonance links
    links: Vec<ResonanceLink>,
    /// Maximum number of nodes to keep
    max_nodes: usize,
    /// Counter for deep mysteries (affects identity)
    mystery_count: u32,
    /// Counter for coherent events
    coherent_count: u32,
}

impl MemoryGraph {
    /// Create a new MemoryGraph
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            links: Vec::new(),
            max_nodes: 500,
            mystery_count: 0,
            coherent_count: 0,
        }
    }

    /// Find associations for a given stimulus context
    /// Returns MemoryEchos for the AttentionField
    pub fn find_associations(&self, context: &StimulusContext) -> Vec<MemoryEcho> {
        let query_sig = MemorySignature::from_context(context, 0.5, 0.0, 0.0);
        
        let mut echoes = Vec::new();
        
        for node in self.nodes.values() {
            let similarity = query_sig.similarity(&node.signature);
            
            if similarity > 0.3 {
                // Calculate resonance based on similarity and node strength
                let node_strength = (node.reinforcement_count as f32).ln_1p() * 0.1;
                let resonance = similarity * (0.5 + node_strength);
                
                echoes.push(MemoryEcho {
                    memory_id: node.id,
                    resonance_score: resonance.min(1.0),
                    similarity,
                });
            }
        }
        
        // Sort by resonance (highest first)
        echoes.sort_by(|a, b| b.resonance_score.partial_cmp(&a.resonance_score).unwrap());
        
        // Limit to top 5
        echoes.truncate(5);
        
        echoes
    }

    /// Get MemoryContext for vehicles (simplified view of relevant history)
    pub fn get_memory_context(&self, context: &StimulusContext) -> MemoryContext {
        let echoes = self.find_associations(context);
        
        if echoes.is_empty() {
            return MemoryContext::default();
        }
        
        // Calculate historical alignment (average similarity of top matches)
        let historical_alignment: f32 = echoes.iter()
            .map(|e| e.similarity)
            .sum::<f32>() / echoes.len() as f32;
        
        // Calculate past outcome valence (average outcome of similar situations)
        let past_outcome_valence: f32 = echoes.iter()
            .filter_map(|e| self.nodes.get(&e.memory_id))
            .map(|n| n.outcome_valence)
            .sum::<f32>() / echoes.len().max(1) as f32;
        
        // Familiarity based on how many matches we found
        let familiarity = (echoes.len() as f32 / 5.0).min(1.0);
        
        MemoryContext {
            historical_alignment,
            past_outcome_valence: (past_outcome_valence + 1.0) / 2.0,  // Normalize to 0-1
            familiarity,
        }
    }

    /// Record a coherent event (write to memory graph)
    /// Only called when Observer-Witness classifies outcome as Coherent
    pub fn record_coherent_event(
        &mut self,
        context: &StimulusContext,
        coherence: f32,
        dissonance: f32,
        valence: f32,
        action: Option<MotorImpulse>,
    ) {
        let signature = MemorySignature::from_context(context, coherence, dissonance, valence);
        
        // Check if similar memory exists
        if let Some(existing_id) = self.find_similar_node(&signature) {
            // Reinforce existing memory
            self.reinforce_node(&existing_id, valence, action);
        } else {
            // Create new memory node
            let node = MemoryNode {
                id: Uuid::new_v4(),
                signature,
                first_observed: Instant::now(),
                last_reinforced: Instant::now(),
                reinforcement_count: 1,
                outcome_valence: valence,
                outcome_action: action,
                is_mystery: false,
                karmic_weight: 0.0,  // Neutral by default, updated via crystallize_experience
                location: (0, 0, 0),  // Default location, should be set via crystallize_experience
            };
            
            let node_id = node.id;
            self.nodes.insert(node_id, node);
            
            // Create links to similar nodes
            self.create_links_for_node(node_id);
        }
        
        self.coherent_count += 1;
        self.enforce_limits();
    }

    /// Record a deep mystery (unresolved but meaningful)
    /// Only called when Observer-Witness classifies outcome as DeepMystery
    pub fn record_deep_mystery(
        &mut self,
        context: &StimulusContext,
        coherence: f32,
        dissonance: f32,
        valence: f32,
    ) {
        let signature = MemorySignature::from_context(context, coherence, dissonance, valence);
        
        // Mysteries are always new nodes (they're unique)
        let node = MemoryNode {
            id: Uuid::new_v4(),
            signature,
            first_observed: Instant::now(),
            last_reinforced: Instant::now(),
            reinforcement_count: 1,
            outcome_valence: valence,
            outcome_action: None,
            is_mystery: true,
            karmic_weight: 0.0,  // Mysteries are ethically neutral (uncertain)
            location: (0, 0, 0),  // Default location
        };
        
        let node_id = node.id;
        self.nodes.insert(node_id, node);
        
        // Mysteries create weaker links (they're uncertain)
        // But they still connect to the graph
        self.create_links_for_node(node_id);
        
        self.mystery_count += 1;
        self.enforce_limits();
    }

    /// Reinforce an existing memory node
    fn reinforce_node(&mut self, node_id: &Uuid, new_valence: f32, action: Option<MotorImpulse>) {
        if let Some(node) = self.nodes.get_mut(node_id) {
            node.reinforcement_count += 1;
            node.last_reinforced = Instant::now();
            
            // Update valence with moving average
            node.outcome_valence = node.outcome_valence * 0.9 + new_valence * 0.1;
            
            // Update action if provided
            if action.is_some() {
                node.outcome_action = action;
            }
        }
        
        // Also reinforce connected links
        for link in &mut self.links {
            if link.from == *node_id || link.to == *node_id {
                link.strength = (link.strength + 0.05).min(1.0);
                link.last_reinforced = Instant::now();
            }
        }
    }

    /// Reinforce a specific link between nodes
    pub fn reinforce_link(&mut self, from: &Uuid, to: &Uuid) {
        for link in &mut self.links {
            if (link.from == *from && link.to == *to) || (link.from == *to && link.to == *from) {
                link.strength = (link.strength + 0.1).min(1.0);
                link.last_reinforced = Instant::now();
                return;
            }
        }
    }

    /// Find a similar node in the graph
    fn find_similar_node(&self, signature: &MemorySignature) -> Option<Uuid> {
        let threshold = 0.7;
        
        for (id, node) in &self.nodes {
            if signature.similarity(&node.signature) > threshold {
                return Some(*id);
            }
        }
        
        None
    }

    /// Create links from a new node to existing similar nodes
    fn create_links_for_node(&mut self, node_id: Uuid) {
        let node = match self.nodes.get(&node_id) {
            Some(n) => n.clone(),
            None => return,
        };
        
        // Find similar nodes and create links
        for (other_id, other_node) in &self.nodes {
            if *other_id == node_id {
                continue;
            }
            
            let similarity = node.signature.similarity(&other_node.signature);
            
            if similarity > 0.3 {
                // Determine link type
                let resonance_type = if (node.outcome_valence - other_node.outcome_valence).abs() < 0.3 {
                    ResonanceType::Outcome
                } else {
                    ResonanceType::Situational
                };
                
                // Create link with strength based on similarity
                self.links.push(ResonanceLink {
                    from: node_id,
                    to: *other_id,
                    strength: similarity * 0.5,
                    last_reinforced: Instant::now(),
                    resonance_type,
                });
            }
        }
    }

    /// Decay all links (call periodically)
    pub fn decay_links(&mut self) {
        let now = Instant::now();
        
        // Decay links that haven't been reinforced recently
        for link in &mut self.links {
            let age = now.duration_since(link.last_reinforced);
            if age > DECAY_GRACE_PERIOD {
                link.strength = (link.strength - DECAY_RATE).max(0.0);
            }
        }
        
        // Remove dead links
        self.links.retain(|link| link.strength > MIN_LINK_STRENGTH);
    }

    /// Enforce node and link limits
    fn enforce_limits(&mut self) {
        // Remove oldest, weakest nodes if over limit
        while self.nodes.len() > self.max_nodes {
            // Find the weakest node (lowest reinforcement, oldest)
            let weakest = self.nodes.iter()
                .min_by(|a, b| {
                    let score_a = a.1.reinforcement_count as f32;
                    let score_b = b.1.reinforcement_count as f32;
                    score_a.partial_cmp(&score_b).unwrap()
                })
                .map(|(id, _)| *id);
            
            if let Some(id) = weakest {
                self.nodes.remove(&id);
                // Remove associated links
                self.links.retain(|link| link.from != id && link.to != id);
            }
        }
    }

    /// Get the number of nodes
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Get the number of links
    pub fn link_count(&self) -> usize {
        self.links.len()
    }

    /// Get mystery ratio (how much of our memory is unresolved?)
    pub fn mystery_ratio(&self) -> f32 {
        if self.coherent_count + self.mystery_count == 0 {
            return 0.0;
        }
        self.mystery_count as f32 / (self.coherent_count + self.mystery_count) as f32
    }

    /// Check if a situation is familiar (for identity vehicle)
    pub fn is_familiar(&self, context: &StimulusContext) -> bool {
        !self.find_associations(context).is_empty()
    }

    /// Get outcome prediction for a similar situation
    pub fn predict_outcome(&self, context: &StimulusContext) -> Option<f32> {
        let echoes = self.find_associations(context);
        
        if echoes.is_empty() {
            return None;
        }
        
        // Weighted average of outcomes by similarity
        let total_weight: f32 = echoes.iter().map(|e| e.similarity).sum();
        let weighted_outcome: f32 = echoes.iter()
            .filter_map(|e| self.nodes.get(&e.memory_id))
            .zip(echoes.iter())
            .map(|(n, e)| n.outcome_valence * e.similarity)
            .sum();
        
        Some(weighted_outcome / total_weight)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // AXIOM 4: REFLECTION - Ethical Crystallization
    // ═══════════════════════════════════════════════════════════════════════

    /// Crystallize an experience with ethical weighting
    /// 
    /// This is where "Memory as Being" is implemented. The agent doesn't just
    /// "know" what happened - the memory acts as a permanent deformation in
    /// the Sanctuary field with ethical resistance properties.
    /// 
    /// # Arguments
    /// * `context` - The stimulus context that was experienced
    /// * `consent_result` - The ethical outcome of the action
    /// * `location` - Where in Sanctuary this occurred (x, y, z)
    /// * `action` - The motor action that was taken (if any)
    /// 
    /// # Physics
    /// - ConsentResult::Allowed → karmic_weight = +efficiency_score (Flow)
    /// - ConsentResult::Denied → karmic_weight = -0.5 (Moderate resistance)
    /// - ConsentResult::Violation → karmic_weight = -1.0 (Maximum resistance/wall)
    /// - ConsentResult::Neutral → karmic_weight = 0.0 (No ethical dimension)
    pub fn crystallize_experience(
        &mut self,
        context: &StimulusContext,
        consent_result: ConsentResult,
        location: (i32, i32, i32),
        action: Option<MotorImpulse>,
    ) {
        // Calculate karmic weight from consent result
        let karmic_weight = match consent_result {
            ConsentResult::Allowed { efficiency_score } => {
                // Positive resonance - ethical flow
                // Scale efficiency_score (0.0-1.0) to karmic weight
                efficiency_score.min(1.0).max(0.0)
            }
            ConsentResult::Denied => {
                // Moderate negative - ethical friction
                -0.5
            }
            ConsentResult::Violation => {
                // Maximum negative - ethical wall
                -1.0
            }
            ConsentResult::Neutral => {
                // No ethical dimension
                0.0
            }
        };

        // Calculate valence from karmic weight
        let valence = karmic_weight;

        // Extract triune state from context (if available)
        let coherence = context.contextual_salience;
        let dissonance = 0.0;  // Could be calculated from context

        let signature = MemorySignature::from_context(context, coherence, dissonance, valence);
        
        // Check if similar memory exists
        if let Some(existing_id) = self.find_similar_node(&signature) {
            // Reinforce existing memory and update its karmic weight
            if let Some(node) = self.nodes.get_mut(&existing_id) {
                node.reinforcement_count += 1;
                node.last_reinforced = Instant::now();
                
                // Update karmic weight with moving average (90% old, 10% new)
                // This allows patterns to shift over time
                let weight = 0.1;
                node.karmic_weight = node.karmic_weight * (1.0 - weight) + karmic_weight * weight;
                
                // Fix 1: Update outcome_valence (Predictive Model)
                // Without this, ethical memories decay and predictive models stagnate
                node.outcome_valence = node.outcome_valence * (1.0 - weight) + karmic_weight * weight;
                
                // Update location (use most recent)
                node.location = location;
                
                // Update action if provided
                if action.is_some() {
                    node.outcome_action = action;
                }
            }

            // Fix 2: Reinforce connected edges (Hebbian Learning)
            // Without this, ResonanceLinks between memories never strengthen
            for link in &mut self.links {
                if link.from == existing_id || link.to == existing_id {
                    link.strength = (link.strength + 0.05).min(1.0);
                    link.last_reinforced = Instant::now();
                }
            }
        } else {
            // Create new memory node with ethical dimension
            let node = MemoryNode {
                id: Uuid::new_v4(),
                signature,
                first_observed: Instant::now(),
                last_reinforced: Instant::now(),
                reinforcement_count: 1,
                outcome_valence: valence,
                outcome_action: action,
                is_mystery: false,
                karmic_weight,
                location,
            };
            
            let node_id = node.id;
            self.nodes.insert(node_id, node);
            
            // Create links to similar nodes
            self.create_links_for_node(node_id);
        }
        
        self.coherent_count += 1;
        self.enforce_limits();
    }

    /// Query memories near a spatial location in Sanctuary
    /// 
    /// Returns memories within a given radius, with their karmic weights.
    /// This is used by Sanctuary to calculate local ethical resistance.
    /// 
    /// # Arguments
    /// * `location` - Center point (x, y, z)
    /// * `radius` - Search radius (in voxel units)
    /// 
    /// # Returns
    /// Vector of (memory_id, karmic_weight, distance) tuples
    pub fn query_nearby_memories(
        &self,
        location: (i32, i32, i32),
        radius: i32,
    ) -> Vec<(Uuid, f32, f32)> {
        let mut nearby = Vec::new();
        let (cx, cy, cz) = location;
        
        for (id, node) in &self.nodes {
            let (nx, ny, nz) = node.location;
            
            // Calculate 3D Euclidean distance
            let dx = (nx - cx) as f32;
            let dy = (ny - cy) as f32;
            let dz = (nz - cz) as f32;
            let distance = (dx * dx + dy * dy + dz * dz).sqrt();
            
            if distance <= radius as f32 {
                nearby.push((*id, node.karmic_weight, distance));
            }
        }
        
        // Sort by distance (closest first)
        nearby.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap());
        
        nearby
    }

    /// Calculate the local karmic field strength at a location
    /// 
    /// This aggregates the karmic weights of nearby memories, weighted by
    /// inverse distance and recency. Returns a value in [-1.0, 1.0] where:
    /// - Negative values indicate ethical resistance (past violations)
    /// - Positive values indicate ethical flow (past resonance)
    /// - Zero indicates neutral/unknown ethical terrain
    /// 
    /// # Arguments
    /// * `location` - Query location (x, y, z)
    /// * `radius` - Influence radius for memories
    /// 
    /// # Returns
    /// Local karmic field strength
    pub fn calculate_local_karma(
        &self,
        location: (i32, i32, i32),
        radius: i32,
    ) -> f32 {
        let nearby = self.query_nearby_memories(location, radius);
        
        if nearby.is_empty() {
            return 0.0;  // No history = neutral
        }
        
        let mut weighted_sum = 0.0f32;
        let mut weight_total = 0.0f32;
        
        for (_id, karmic_weight, distance) in nearby {
            // Weight by inverse distance (closer = stronger influence)
            // Add 1.0 to avoid division by zero at distance=0
            let distance_weight = 1.0 / (distance + 1.0);
            
            // Could also weight by recency here if we track timestamps
            let weight = distance_weight;
            
            weighted_sum += karmic_weight * weight;
            weight_total += weight;
        }
        
        if weight_total > 0.0 {
            (weighted_sum / weight_total).max(-1.0).min(1.0)
        } else {
            0.0
        }
    }
}

impl Default for MemoryGraph {
    fn default() -> Self {
        Self::new()
    }
}

