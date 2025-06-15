// AIDEV-NOTE: Database layer for provenance event persistence using SQLite
use std::collections::HashMap;
use sqlx::{Row, SqlitePool};

use crate::{ProvenanceEvent, EventResponse, ManifestData};

// AIDEV-NOTE: Database connection pool wrapper
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn new(url: &str) -> Result<Self, String> {
        let pool = SqlitePool::connect(url).await.map_err(|e| e.to_string())?;
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS events (\n  id INTEGER PRIMARY KEY AUTOINCREMENT,\n  timestamp TEXT NOT NULL,\n  event_type TEXT NOT NULL,\n  text_hash TEXT NOT NULL,\n  source TEXT,\n  span_length INTEGER\n)"
        )
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;
        Ok(Self { pool })
    }

    // AIDEV-NOTE: Stores provenance event and returns generated ID
    pub async fn insert_event(&self, event: ProvenanceEvent) -> Result<EventResponse, String> {
        let result = sqlx::query(
            "INSERT INTO events (timestamp, event_type, text_hash, source, span_length) VALUES (?1, ?2, ?3, ?4, ?5)"
        )
        .bind(&event.timestamp)
        .bind(&event.event_type)
        .bind(&event.text_hash)
        .bind(&event.source)
        .bind(event.span_length as i64)
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(EventResponse {
            id: result.last_insert_rowid(),
            text_hash: event.text_hash,
        })
    }

    // AIDEV-NOTE: Retrieves filtered event history with optional pagination
    pub async fn get_events(
        &self,
        limit: Option<u32>,
        event_type: Option<String>,
    ) -> Result<Vec<ProvenanceEvent>, String> {
        let mut query = String::from(
            "SELECT timestamp, event_type, text_hash, source, span_length FROM events",
        );
        if event_type.is_some() {
            query.push_str(" WHERE event_type = ?1");
        }
        query.push_str(" ORDER BY timestamp DESC");
        if let Some(l) = limit {
            query.push_str(&format!(" LIMIT {}", l));
        }

        let mut q = sqlx::query(&query);
        if let Some(t) = &event_type {
            q = q.bind(t);
        }

        let rows = q.fetch_all(&self.pool).await.map_err(|e| e.to_string())?;
        let events = rows
            .into_iter()
            .map(|row| ProvenanceEvent {
                timestamp: row.get::<String, _>("timestamp"),
                event_type: row.get::<String, _>("event_type"),
                text_hash: row.get::<String, _>("text_hash"),
                source: row.get::<String, _>("source"),
                span_length: row.get::<i64, _>("span_length") as usize,
            })
            .collect();
        Ok(events)
    }

    // AIDEV-NOTE: Generates manifest data with statistics from all stored events
    pub async fn generate_manifest(&self) -> Result<ManifestData, String> {
        let events = self.get_events(None, None).await?;
        
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
    pub async fn clear_events(&self) -> Result<(), String> {
        sqlx::query("DELETE FROM events")
            .execute(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    // AIDEV-NOTE: Get event count by type for analytics
    pub async fn get_event_counts(&self) -> Result<HashMap<String, usize>, String> {
        let rows = sqlx::query("SELECT event_type, COUNT(*) as count FROM events GROUP BY event_type")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        let mut counts = HashMap::new();
        for row in rows {
            counts.insert(
                row.get::<String, _>("event_type"),
                row.get::<i64, _>("count") as usize,
            );
        }
        Ok(counts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const TEST_DB_URL: &str = "sqlite::memory:";

    fn create_test_event(event_type: &str, source: &str, span_length: usize) -> ProvenanceEvent {
        ProvenanceEvent {
            timestamp: chrono::Utc::now().to_rfc3339(),
            event_type: event_type.to_string(),
            text: format!("hash_{}", source),
            source: source.to_string(),
            span_length,
        }
    }

    #[tokio::test]
    async fn test_insert_event() {
        let db = Database::new(TEST_DB_URL).await.unwrap();
        db.clear_events().await.unwrap();

        let event = create_test_event("human", "user", 10);
        let result = db.insert_event(event.clone()).await;
        
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.text_hash, event.text);
        assert_eq!(response.id, 1);
    }

    #[tokio::test]
    async fn test_get_events_with_filter() {
        let db = Database::new(TEST_DB_URL).await.unwrap();
        db.clear_events().await.unwrap();
        
        // Insert different types of events
        db.insert_event(create_test_event("human", "user", 10)).await.unwrap();
        db.insert_event(create_test_event("ai", "gpt-4", 15)).await.unwrap();
        db.insert_event(create_test_event("cited", "wikipedia", 20)).await.unwrap();
        
        // Test filtering by type
        let human_events = db.get_events(None, Some("human".to_string())).await.unwrap();
        assert_eq!(human_events.len(), 1);
        assert_eq!(human_events[0].event_type, "human");
        
        // Test no filter
        let all_events = db.get_events(None, None).await.unwrap();
        assert_eq!(all_events.len(), 3);
    }

    #[tokio::test]
    async fn test_get_events_with_limit() {
        let db = Database::new(TEST_DB_URL).await.unwrap();
        db.clear_events().await.unwrap();
        
        // Insert multiple events
        for i in 0..5 {
            db.insert_event(create_test_event("human", &format!("user{}", i), 10)).await.unwrap();
        }
        
        let limited_events = db.get_events(Some(3), None).await.unwrap();
        assert_eq!(limited_events.len(), 3);
    }

    #[tokio::test]
    async fn test_generate_manifest() {
        let db = Database::new(TEST_DB_URL).await.unwrap();
        db.clear_events().await.unwrap();
        
        // Insert events with known character counts
        db.insert_event(create_test_event("human", "user", 60)).await.unwrap();
        db.insert_event(create_test_event("ai", "gpt-4", 30)).await.unwrap();
        db.insert_event(create_test_event("cited", "wikipedia", 10)).await.unwrap();
        
        let manifest = db.generate_manifest().await.unwrap();
        
        assert_eq!(manifest.human_percentage, 60.0);
        assert_eq!(manifest.ai_percentage, 30.0);
        assert_eq!(manifest.cited_percentage, 10.0);
        assert_eq!(manifest.total_characters, 100);
    }

    #[tokio::test]
    async fn test_get_event_counts() {
        let db = Database::new(TEST_DB_URL).await.unwrap();
        db.clear_events().await.unwrap();
        
        // Insert multiple events of different types
        db.insert_event(create_test_event("human", "user1", 10)).await.unwrap();
        db.insert_event(create_test_event("human", "user2", 10)).await.unwrap();
        db.insert_event(create_test_event("ai", "gpt-4", 15)).await.unwrap();
        
        let counts = db.get_event_counts().await.unwrap();
        
        assert_eq!(counts.get("human"), Some(&2));
        assert_eq!(counts.get("ai"), Some(&1));
        assert_eq!(counts.get("cited"), None);
    }

    #[tokio::test]
    async fn test_clear_events() {
        let db = Database::new(TEST_DB_URL).await.unwrap();
        
        // Insert some events
        db.insert_event(create_test_event("human", "user", 10)).await.unwrap();
        
        // Verify events exist
        let events_before = db.get_events(None, None).await.unwrap();
        assert!(!events_before.is_empty());
        
        // Clear events
        db.clear_events().await.unwrap();
        
        // Verify events are cleared
        let events_after = db.get_events(None, None).await.unwrap();
        assert!(events_after.is_empty());
    }
}