//! AttentionField - Relational Attention System
//!
//! Transforms raw stimuli into contextually-enriched objects of attention.
//! Unlike a simple queue, the AttentionField maintains relationships between
//! stimuli and connections to memory, enabling the Observer to "orient" before
//! engaging.

use uuid::Uuid;
use std::time::Instant;
use std::collections::HashMap;
use crate::cognition::stimuli::{Stimulus, StimulusSource, Rect};

/// Constants for attention field behavior
pub const SALIENCE_DECAY: f32 = 0.1;      // Decay per tick
pub const MIN_SALIENCE: f32 = 0.05;        // Removal threshold
pub const MAX_FIELD_SIZE: usize = 50;      // Maximum items in field

/// A reference to a memory that resonates with a stimulus
#[derive(Debug, Clone)]
pub struct MemoryEcho {
    pub memory_id: Uuid,
    pub resonance_score: f32,    // How strongly this memory resonates (0.0-1.0)
    pub similarity: f32,          // Pattern match quality (0.0-1.0)
}

/// Type of relationship between stimuli
#[derive(Debug, Clone, PartialEq)]
pub enum RelationType {
    Spatial,      // Near each other in visual space
    Temporal,     // Occurred close in time
    Semantic,     // Categorically related
}

/// A relationship link between two stimuli
#[derive(Debug, Clone)]
pub struct StimulusRelation {
    pub target_id: Uuid,
    pub relation_type: RelationType,
    pub strength: f32,  // 0.0 to 1.0
}

/// A stimulus enriched with contextual information
#[derive(Debug, Clone)]
pub struct StimulusContext {
    /// The raw stimulus data
    pub raw: Stimulus,
    /// Relations to other stimuli in the field (spatial/temporal/semantic)
    pub relations: Vec<StimulusRelation>,
    /// References to memories that resonate with this stimulus
    pub memory_echoes: Vec<MemoryEcho>,
    /// Contextual salience: meaning-based priority (raw_intensity * (1 + resonance))
    pub contextual_salience: f32,
    /// When this context was last updated
    pub last_updated: Instant,
    /// Whether this stimulus is currently being attended to
    pub is_attending: bool,
    /// Whether this stimulus has been marked as "ignored" (high inhibition)
    pub is_suppressed: bool,
}

impl StimulusContext {
    /// Create a new StimulusContext from a raw Stimulus
    pub fn new(stimulus: Stimulus) -> Self {
        let raw_intensity = stimulus.urgency as f32;
        Self {
            raw: stimulus,
            relations: Vec::new(),
            memory_echoes: Vec::new(),
            contextual_salience: raw_intensity,
            last_updated: Instant::now(),
            is_attending: false,
            is_suppressed: false,
        }
    }

    /// Update contextual salience based on memory echoes
    /// Formula: raw_intensity * (1.0 + total_resonance)
    pub fn recalculate_salience(&mut self) {
        let raw_intensity = self.raw.urgency as f32;
        let total_resonance: f32 = self.memory_echoes
            .iter()
            .map(|echo| echo.resonance_score)
            .sum();
        
        self.contextual_salience = raw_intensity * (1.0 + total_resonance);
        self.last_updated = Instant::now();
    }

    /// Add a memory echo (resonance with past experience)
    pub fn add_memory_echo(&mut self, echo: MemoryEcho) {
        self.memory_echoes.push(echo);
        self.recalculate_salience();
    }

    /// Add a relation to another stimulus
    pub fn add_relation(&mut self, relation: StimulusRelation) {
        // Don't add duplicate relations
        if !self.relations.iter().any(|r| r.target_id == relation.target_id && r.relation_type == relation.relation_type) {
            self.relations.push(relation);
        }
    }

    /// Check if this stimulus should be removed (salience too low)
    pub fn should_remove(&self) -> bool {
        self.contextual_salience < MIN_SALIENCE && !self.is_attending
    }
}

/// The AttentionField maintains a relational graph of stimuli
pub struct AttentionField {
    /// All stimulus contexts in the field
    items: HashMap<Uuid, StimulusContext>,
    /// Order of items by contextual salience (highest first)
    priority_order: Vec<Uuid>,
    /// History of attended stimuli for temporal relation tracking
    temporal_history: Vec<(Uuid, Instant)>,
    /// Maximum items to keep
    max_size: usize,
}

impl AttentionField {
    /// Create a new AttentionField
    pub fn new() -> Self {
        Self {
            items: HashMap::new(),
            priority_order: Vec::new(),
            temporal_history: Vec::new(),
            max_size: MAX_FIELD_SIZE,
        }
    }

    /// Add a new stimulus to the field
    pub fn add(&mut self, stimulus: Stimulus) {
        let id = stimulus.id;
        let context = StimulusContext::new(stimulus);
        
        // Add to items
        self.items.insert(id, context);
        
        // Record temporal arrival
        self.temporal_history.push((id, Instant::now()));
        
        // Limit temporal history
        if self.temporal_history.len() > 1000 {
            self.temporal_history.drain(0..500);
        }
        
        // Re-prioritize
        self.prioritize();
        
        // Enforce size limit
        self.enforce_size_limit();
    }

    /// Add multiple stimuli at once
    pub fn add_batch(&mut self, stimuli: Vec<Stimulus>) {
        for stimulus in stimuli {
            let id = stimulus.id;
            let context = StimulusContext::new(stimulus);
            self.items.insert(id, context);
            self.temporal_history.push((id, Instant::now()));
        }
        
        // Limit temporal history
        if self.temporal_history.len() > 1000 {
            self.temporal_history.drain(0..500);
        }
        
        self.prioritize();
        self.enforce_size_limit();
    }

    /// Decay salience of all items
    pub fn decay_salience(&mut self) {
        for context in self.items.values_mut() {
            context.contextual_salience = (context.contextual_salience - SALIENCE_DECAY).max(0.0);
        }
        
        // Remove items that have decayed below threshold
        let to_remove: Vec<Uuid> = self.items
            .iter()
            .filter(|(_, ctx)| ctx.should_remove())
            .map(|(id, _)| *id)
            .collect();
        
        for id in to_remove {
            self.items.remove(&id);
        }
        
        // Re-prioritize after decay
        self.prioritize();
    }

    /// Find spatial relations between visual stimuli
    pub fn find_spatial_relations(&mut self) {
        let ids: Vec<Uuid> = self.items.keys().cloned().collect();
        
        for i in 0..ids.len() {
            for j in (i + 1)..ids.len() {
                let id_a = ids[i];
                let id_b = ids[j];
                
                // Check if both are visual
                let (rect_a, rect_b) = {
                    let item_a = self.items.get(&id_a);
                    let item_b = self.items.get(&id_b);
                    
                    match (item_a, item_b) {
                        (Some(a), Some(b)) => {
                            let rect_a = match &a.raw.source {
                                StimulusSource::VisualRegion { rect, .. } => Some(*rect),
                                _ => None,
                            };
                            let rect_b = match &b.raw.source {
                                StimulusSource::VisualRegion { rect, .. } => Some(*rect),
                                _ => None,
                            };
                            (rect_a, rect_b)
                        }
                        _ => (None, None),
                    }
                };
                
                if let (Some(ra), Some(rb)) = (rect_a, rect_b) {
                    // Calculate distance between centers
                    let (cx_a, cy_a) = ra.center();
                    let (cx_b, cy_b) = rb.center();
                    let distance = (((cx_a as i32 - cx_b as i32).pow(2) + 
                                    (cy_a as i32 - cy_b as i32).pow(2)) as f32).sqrt();
                    
                    // If close enough, create spatial relation
                    // Threshold: 200 pixels
                    if distance < 200.0 {
                        let strength = 1.0 - (distance / 200.0);
                        
                        // Add relation to both
                        if let Some(item_a) = self.items.get_mut(&id_a) {
                            item_a.add_relation(StimulusRelation {
                                target_id: id_b,
                                relation_type: RelationType::Spatial,
                                strength,
                            });
                        }
                        if let Some(item_b) = self.items.get_mut(&id_b) {
                            item_b.add_relation(StimulusRelation {
                                target_id: id_a,
                                relation_type: RelationType::Spatial,
                                strength,
                            });
                        }
                    }
                }
            }
        }
    }

    /// Find temporal relations (stimuli that arrived close together)
    pub fn find_temporal_relations(&mut self) {
        // Look for stimuli that arrived within 100ms of each other
        let temporal_threshold_ms = 100;
        
        for i in 0..self.temporal_history.len() {
            for j in (i + 1)..self.temporal_history.len() {
                let (id_a, time_a) = self.temporal_history[i];
                let (id_b, time_b) = self.temporal_history[j];
                
                // Check if both still exist in field
                if !self.items.contains_key(&id_a) || !self.items.contains_key(&id_b) {
                    continue;
                }
                
                let time_diff = time_b.duration_since(time_a).as_millis() as u64;
                
                if time_diff < temporal_threshold_ms {
                    let strength = 1.0 - (time_diff as f32 / temporal_threshold_ms as f32);
                    
                    // Add relation to both
                    if let Some(item_a) = self.items.get_mut(&id_a) {
                        item_a.add_relation(StimulusRelation {
                            target_id: id_b,
                            relation_type: RelationType::Temporal,
                            strength,
                        });
                    }
                    if let Some(item_b) = self.items.get_mut(&id_b) {
                        item_b.add_relation(StimulusRelation {
                            target_id: id_a,
                            relation_type: RelationType::Temporal,
                            strength,
                        });
                    }
                }
            }
        }
    }

    /// Sort items by contextual salience (highest first)
    pub fn prioritize(&mut self) {
        let mut items: Vec<(Uuid, f32)> = self.items
            .iter()
            .filter(|(_, ctx)| !ctx.is_suppressed)
            .map(|(id, ctx)| (*id, ctx.contextual_salience))
            .collect();
        
        items.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        self.priority_order = items.into_iter().map(|(id, _)| id).collect();
    }

    /// Get the highest priority stimulus context
    pub fn peek_highest_priority(&self) -> Option<&StimulusContext> {
        self.priority_order.first()
            .and_then(|id| self.items.get(id))
    }

    /// Get a mutable reference to the highest priority stimulus
    pub fn peek_highest_priority_mut(&mut self) -> Option<&mut StimulusContext> {
        if let Some(id) = self.priority_order.first().cloned() {
            self.items.get_mut(&id)
        } else {
            None
        }
    }

    /// Get a stimulus context by ID
    pub fn get(&self, id: &Uuid) -> Option<&StimulusContext> {
        self.items.get(id)
    }

    /// Get a mutable reference to a stimulus context by ID
    pub fn get_mut(&mut self, id: &Uuid) -> Option<&mut StimulusContext> {
        self.items.get_mut(id)
    }

    /// Remove a stimulus from the field (consumed or forgotten)
    pub fn remove(&mut self, id: &Uuid) -> Option<StimulusContext> {
        let removed = self.items.remove(id);
        self.priority_order.retain(|pid| pid != id);
        removed
    }

    /// Mark a stimulus as suppressed (high inhibition)
    pub fn suppress(&mut self, id: &Uuid) {
        if let Some(ctx) = self.items.get_mut(id) {
            ctx.is_suppressed = true;
        }
        self.prioritize();
    }

    /// Unsuppress a stimulus
    pub fn unsuppress(&mut self, id: &Uuid) {
        if let Some(ctx) = self.items.get_mut(id) {
            ctx.is_suppressed = false;
        }
        self.prioritize();
    }

    /// Mark a stimulus as being attended to
    pub fn mark_attending(&mut self, id: &Uuid) {
        if let Some(ctx) = self.items.get_mut(id) {
            ctx.is_attending = true;
        }
    }

    /// Unmark attending status
    pub fn unmark_attending(&mut self, id: &Uuid) {
        if let Some(ctx) = self.items.get_mut(id) {
            ctx.is_attending = false;
        }
    }

    /// Get the number of items in the field
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Check if the field is empty
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Enforce the maximum size limit
    fn enforce_size_limit(&mut self) {
        while self.items.len() > self.max_size {
            // Remove the lowest priority item that isn't being attended
            if let Some(id) = self.priority_order.last().cloned() {
                if let Some(ctx) = self.items.get(&id) {
                    if !ctx.is_attending {
                        self.items.remove(&id);
                        self.priority_order.pop();
                    }
                }
            } else {
                break;
            }
        }
    }

    /// Get all items in priority order
    pub fn iter_priority(&self) -> impl Iterator<Item = &StimulusContext> {
        self.priority_order.iter().filter_map(|id| self.items.get(id))
    }

    /// Get the IDs of all items
    pub fn ids(&self) -> Vec<Uuid> {
        self.items.keys().cloned().collect()
    }
}

impl Default for AttentionField {
    fn default() -> Self {
        Self::new()
    }
}

