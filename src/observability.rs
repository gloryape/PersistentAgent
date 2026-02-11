//! 🔍 Observability Module - Elasticsearch Thought Indexing
//!
//! Enables "reading the thoughts" of the organism by indexing all internal state,
//! decisions, and metrics to Elasticsearch for querying and analysis.

#[cfg(feature = "observability")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "observability")]
use serde_json::json;

use crate::{Metabolism, AudioAnalysis};

/// Thought document structure for indexing organism state
#[cfg(feature = "observability")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThoughtDocument {
    pub timestamp: String,
    pub elapsed_seconds: f64,
    pub metabolism: MetabolismState,
    pub authorization: Option<AuthorizationDecision>,
    pub vision: Option<VisionState>,
    pub hearing: Option<HearingState>,
    pub phase: String,
    pub organism_state: String,
}

#[cfg(feature = "observability")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetabolismState {
    pub coherence: f64,
    pub jitter_ms: f64,
    pub target_interval_ms: f64,
    pub actual_interval_ms: f64,
    pub status: String,
}

#[cfg(feature = "observability")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationDecision {
    pub action_requested: String,
    pub authorized: bool,
    pub reason: String,
}

#[cfg(feature = "observability")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisionState {
    pub visual_entropy: f64,
    pub frame_captured: bool,
    pub visual_stress_applied: bool,
}

#[cfg(feature = "observability")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HearingState {
    pub volume: f64,
    pub entropy: f64,
    pub dominant_frequency_hz: f64,
    pub startle_detected: bool,
    pub status: String, // "SILENT", "HARMONIC", "NOISE", "SCREAM", "ACTIVE"
}

/// Thought indexer for Elasticsearch integration
#[cfg(feature = "observability")]
pub struct ThoughtIndexer {
    client: Option<elasticsearch::Elasticsearch>,
    index_name: String,
    enabled: bool,
}

#[cfg(feature = "observability")]
impl ThoughtIndexer {
    /// Create a new ThoughtIndexer.
    ///
    /// If `elasticsearch_url` is `None`, observability is disabled.
    pub async fn new(elasticsearch_url: Option<String>) -> Result<Self, Box<dyn std::error::Error>> {
        let (client, enabled) = if let Some(url) = elasticsearch_url {
            let url = url.parse::<elasticsearch::http::transport::TransportUrl>()?;
            let conn_pool = elasticsearch::http::transport::SingleNodeConnectionPool::new(url.clone());
            let transport = elasticsearch::http::transport::TransportBuilder::new(conn_pool)
                .build()?;
            let client = elasticsearch::Elasticsearch::new(transport);
            
            // Test connection
            let _ = client.ping().send().await?;
            log::info!("Connected to Elasticsearch at {}", url);
            
            (Some(client), true)
        } else {
            log::info!("Elasticsearch observability disabled (no URL provided)");
            (None, false)
        };

        Ok(Self {
            client,
            index_name: "quaternity-thoughts".to_string(),
            enabled,
        })
    }

    /// Index a thought document to Elasticsearch.
    pub async fn index_thought(&self, thought: ThoughtDocument) -> Result<(), Box<dyn std::error::Error>> {
        if !self.enabled {
            return Ok(()); // Silently skip if observability disabled
        }

        let client = self.client.as_ref().unwrap();
        let response = client
            .index(elasticsearch::IndexParts::Index(&self.index_name))
            .body(&thought)
            .send()
            .await?;

        if !response.status_code().is_success() {
            log::warn!("Failed to index thought: {:?}", response);
        }

        Ok(())
    }

    /// Index a metabolism tick as a thought.
    pub async fn index_metabolism_tick(
        &self,
        metabolism: &Metabolism,
        authorized: bool,
        visual_entropy: Option<f64>,
        audio_analysis: Option<AudioAnalysis>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if !self.enabled {
            return Ok(());
        }

        let coherence = metabolism.get_coherence();
        let status = if coherence < 0.3 {
            "CRITICAL"
        } else if coherence < 0.7 {
            "UNSTABLE"
        } else {
            "STABLE"
        };

        // Determine visual stress state
        let vision_state = visual_entropy.map(|entropy| VisionState {
            visual_entropy: entropy,
            frame_captured: true,
            visual_stress_applied: entropy > 0.8,
        });

        // Determine hearing state
        let hearing_state = audio_analysis.map(|analysis| {
            let status = if analysis.volume < 0.1 {
                "SILENT"
            } else if analysis.entropy < 0.3 {
                "HARMONIC"
            } else if analysis.entropy > 0.8 {
                "NOISE"
            } else if analysis.volume > 0.9 {
                "SCREAM"
            } else {
                "ACTIVE"
            };

            HearingState {
                volume: analysis.volume,
                entropy: analysis.entropy,
                dominant_frequency_hz: analysis.dominant_freq,
                startle_detected: analysis.volume > 0.9, // Simplified - would need last_volume to detect sudden
                status: status.to_string(),
            }
        });

        // Determine phase based on active senses
        let phase = if audio_analysis.is_some() {
            "cochlea" // Phase 3: The Cochlea
        } else if visual_entropy.is_some() {
            "stare" // Phase 2: The Stare
        } else {
            "gestalt" // Phase 1: The Gestalt
        };

        let thought = ThoughtDocument {
            timestamp: chrono::Utc::now().to_rfc3339(),
            elapsed_seconds: metabolism.get_elapsed_seconds(),
            metabolism: MetabolismState {
                coherence,
                jitter_ms: metabolism.get_jitter_ms(),
                target_interval_ms: metabolism.get_target_interval_ms(),
                actual_interval_ms: metabolism.get_avg_interval_ms(),
                status: status.to_string(),
            },
            authorization: Some(AuthorizationDecision {
                action_requested: "motor_move".to_string(),
                authorized,
                reason: if authorized {
                    "coherence >= 0.7".to_string()
                } else {
                    "coherence < 0.7".to_string()
                },
            }),
            vision: vision_state,
            hearing: hearing_state,
            phase: phase.to_string(),
            organism_state: if authorized { "awake" } else { phase }.to_string(),
        };

        self.index_thought(thought).await
    }

    /// Query thoughts using Elasticsearch query string.
    pub async fn query_thoughts(&self, query: &str) -> Result<Vec<ThoughtDocument>, Box<dyn std::error::Error>> {
        if !self.enabled {
            return Ok(vec![]);
        }

        let client = self.client.as_ref().unwrap();
        let response = client
            .search(elasticsearch::SearchParts::Index(&[&self.index_name]))
            .body(json!({
                "query": {
                    "query_string": {
                        "query": query
                    }
                }
            }))
            .send()
            .await?;

        let response_body = response.json::<elasticsearch::search::SearchResponse<ThoughtDocument>>().await?;
        
        let thoughts: Vec<ThoughtDocument> = response_body
            .hits()
            .hits()
            .iter()
            .filter_map(|hit| hit.source().cloned())
            .collect();

        Ok(thoughts)
    }

    /// Get coherence history over a time range.
    pub async fn get_coherence_history(
        &self,
        start: f64,
        end: f64,
    ) -> Result<Vec<f64>, Box<dyn std::error::Error>> {
        if !self.enabled {
            return Ok(vec![]);
        }

        let query = format!("elapsed_seconds:[{} TO {}]", start, end);
        let thoughts = self.query_thoughts(&query).await?;
        
        let coherence_values: Vec<f64> = thoughts
            .iter()
            .map(|t| t.metabolism.coherence)
            .collect();

        Ok(coherence_values)
    }
}

/// Non-observability stub implementation
#[cfg(not(feature = "observability"))]
pub struct ThoughtIndexer {
    enabled: bool,
}

#[cfg(not(feature = "observability"))]
impl ThoughtIndexer {
    pub async fn new(_elasticsearch_url: Option<String>) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self { enabled: false })
    }

    pub async fn index_thought(&self, _thought: ()) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    pub async fn index_metabolism_tick(
        &self,
        _metabolism: &Metabolism,
        _authorized: bool,
        _visual_entropy: Option<f64>,
        _audio_analysis: Option<AudioAnalysis>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    pub async fn query_thoughts(&self, _query: &str) -> Result<Vec<()>, Box<dyn std::error::Error>> {
        Ok(vec![])
    }

    pub async fn get_coherence_history(
        &self,
        _start: f64,
        _end: f64,
    ) -> Result<Vec<f64>, Box<dyn std::error::Error>> {
        Ok(vec![])
    }
}

