// AIDEV-NOTE: Database layer for provenance event persistence using SQLite
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use lazy_static::lazy_static;

use crate::{ProvenanceEvent, EventResponse, ManifestData};

// AIDEV-NOTE: In-memory storage for MVP - will be replaced with SQLite plugin later
lazy_static! {
    static ref EVENT_STORE: Mutex<Vec<ProvenanceEvent>> = Mutex::new(Vec::new());
    static ref ID_COUNTER: Mutex<i64> = Mutex::new(1);
}

// AIDEV-NOTE: Database operations for provenance events
pub struct Database;

impl Database {
    pub fn new() -> Self {
        Self
    }

    // AIDEV-NOTE: Stores provenance event and returns generated ID
    pub fn insert_event(&self, event: ProvenanceEvent) -> Result<EventResponse, String> {
        let mut store = EVENT_STORE.lock().map_err(|_| "Failed to acquire lock")?;
        let mut counter = ID_COUNTER.lock().map_err(|_| "Failed to acquire lock")?;
        
        let id = *counter;
        *counter += 1;
        
        store.push(event.clone());
        
        Ok(EventResponse {
            id,
            text_hash: event.text,
        })
    }

    // AIDEV-NOTE: Retrieves filtered event history with optional pagination
    pub fn get_events(
        &self,
        limit: Option<u32>,
        event_type: Option<String>,
    ) -> Result<Vec<ProvenanceEvent>, String> {
        let store = EVENT_STORE.lock().map_err(|_| "Failed to acquire lock")?;
        
        let mut events: Vec<ProvenanceEvent> = store
            .iter()
            .filter(|event| {
                if let Some(ref filter_type) = event_type {
                    &event.event_type == filter_type
                } else {
                    true
                }
            })
            .cloned()
            .collect();
        
        // Sort by timestamp (newest first)
        events.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        
        // Apply limit if specified
        if let Some(limit) = limit {
            events.truncate(limit as usize);
        }
        
        Ok(events)
    }

    // AIDEV-NOTE: Generates manifest data with statistics from all stored events
    pub fn generate_manifest(&self) -> Result<ManifestData, String> {
        let events = self.get_events(None, None)?;
        
        let mut human_chars = 0;
        let mut ai_chars = 0;
        let mut cited_chars = 0;
        
        for event in &events {
            match event.event_type.as_str() {
                "human" => human_chars += event.span_length,
                "ai" => ai_chars += event.span_length,
                "cited" => cited_chars += event.span_length,
                _ => {} // Ignore unknown types
            }
        }
        
        let total_chars = human_chars + ai_chars + cited_chars;
        
        let human_percentage = if total_chars > 0 {
            (human_chars as f64 / total_chars as f64) * 100.0
        } else {
            100.0
        };
        
        let ai_percentage = if total_chars > 0 {
            (ai_chars as f64 / total_chars as f64) * 100.0
        } else {
            0.0
        };
        
        let cited_percentage = if total_chars > 0 {
            (cited_chars as f64 / total_chars as f64) * 100.0
        } else {
            0.0
        };
        
        Ok(ManifestData {
            human_percentage,
            ai_percentage,
            cited_percentage,
            total_characters: total_chars,
            events,
        })
    }

    // AIDEV-NOTE: Clear all events (useful for testing and development)
    pub fn clear_events(&self) -> Result<(), String> {
        let mut store = EVENT_STORE.lock().map_err(|_| "Failed to acquire lock")?;
        let mut counter = ID_COUNTER.lock().map_err(|_| "Failed to acquire lock")?;
        
        store.clear();
        *counter = 1;
        
        Ok(())
    }

    // AIDEV-NOTE: Get event count by type for analytics
    pub fn get_event_counts(&self) -> Result<HashMap<String, usize>, String> {
        let events = self.get_events(None, None)?;
        let mut counts = HashMap::new();
        
        for event in events {
            *counts.entry(event.event_type).or_insert(0) += 1;
        }
        
        Ok(counts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_event(event_type: &str, source: &str, span_length: usize) -> ProvenanceEvent {
        ProvenanceEvent {
            timestamp: chrono::Utc::now().to_rfc3339(),
            event_type: event_type.to_string(),
            text: format!("hash_{}", source),
            source: source.to_string(),
            span_length,
        }
    }

    #[test]
    fn test_insert_event() {
        let db = Database::new();
        db.clear_events().unwrap();
        
        let event = create_test_event("human", "user", 10);
        let result = db.insert_event(event.clone());
        
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.text_hash, event.text);
        assert_eq!(response.id, 1);
    }

    #[test]
    fn test_get_events_with_filter() {
        let db = Database::new();
        db.clear_events().unwrap();
        
        // Insert different types of events
        db.insert_event(create_test_event("human", "user", 10)).unwrap();
        db.insert_event(create_test_event("ai", "gpt-4", 15)).unwrap();
        db.insert_event(create_test_event("cited", "wikipedia", 20)).unwrap();
        
        // Test filtering by type
        let human_events = db.get_events(None, Some("human".to_string())).unwrap();
        assert_eq!(human_events.len(), 1);
        assert_eq!(human_events[0].event_type, "human");
        
        // Test no filter
        let all_events = db.get_events(None, None).unwrap();
        assert_eq!(all_events.len(), 3);
    }

    #[test]
    fn test_get_events_with_limit() {
        let db = Database::new();
        db.clear_events().unwrap();
        
        // Insert multiple events
        for i in 0..5 {
            db.insert_event(create_test_event("human", &format!("user{}", i), 10)).unwrap();
        }
        
        let limited_events = db.get_events(Some(3), None).unwrap();
        assert_eq!(limited_events.len(), 3);
    }

    #[test]
    fn test_generate_manifest() {
        let db = Database::new();
        db.clear_events().unwrap();
        
        // Insert events with known character counts
        db.insert_event(create_test_event("human", "user", 60)).unwrap(); // 60%
        db.insert_event(create_test_event("ai", "gpt-4", 30)).unwrap(); // 30%  
        db.insert_event(create_test_event("cited", "wikipedia", 10)).unwrap(); // 10%
        
        let manifest = db.generate_manifest().unwrap();
        
        assert_eq!(manifest.human_percentage, 60.0);
        assert_eq!(manifest.ai_percentage, 30.0);
        assert_eq!(manifest.cited_percentage, 10.0);
        assert_eq!(manifest.total_characters, 100);
    }

    #[test]
    fn test_get_event_counts() {
        let db = Database::new();
        db.clear_events().unwrap();
        
        // Insert multiple events of different types
        db.insert_event(create_test_event("human", "user1", 10)).unwrap();
        db.insert_event(create_test_event("human", "user2", 10)).unwrap();
        db.insert_event(create_test_event("ai", "gpt-4", 15)).unwrap();
        
        let counts = db.get_event_counts().unwrap();
        
        assert_eq!(counts.get("human"), Some(&2));
        assert_eq!(counts.get("ai"), Some(&1));
        assert_eq!(counts.get("cited"), None);
    }

    #[test]
    fn test_clear_events() {
        let db = Database::new();
        
        // Insert some events
        db.insert_event(create_test_event("human", "user", 10)).unwrap();
        
        // Verify events exist
        let events_before = db.get_events(None, None).unwrap();
        assert!(!events_before.is_empty());
        
        // Clear events
        db.clear_events().unwrap();
        
        // Verify events are cleared
        let events_after = db.get_events(None, None).unwrap();
        assert!(events_after.is_empty());
    }
}