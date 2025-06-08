// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::collections::HashMap;
use tauri::{AppHandle, Manager};
use tauri_plugin_sql::{Migration, MigrationKind};

// AIDEV-NOTE: Foundation types - these structs define the entire provenance data model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvenanceEvent {
    pub timestamp: String,
    pub event_type: String,  // "human", "ai", "cited"
    pub text_hash: String,
    pub source: String,
    pub span_length: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventResponse {
    pub id: i64,
    pub text_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestData {
    pub human_percentage: f64,
    pub ai_percentage: f64,
    pub cited_percentage: f64,
    pub total_characters: usize,
    pub events: Vec<ProvenanceEvent>,
}

#[derive(Debug, Deserialize)]
pub struct AIPrompt {
    pub prompt: String,
    pub model: Option<String>,
    pub max_tokens: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct AIResponse {
    pub content: String,
    pub model: String,
    pub token_count: Option<u32>,
}

// AIDEV-NOTE: Schema evolution - versioned migrations ensure data integrity across updates
pub fn create_migrations() -> Vec<Migration> {
    vec![
        Migration {
            version: 1,
            description: "create_events_table",
            sql: r#"
                CREATE TABLE events (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    timestamp TEXT NOT NULL,
                    event_type TEXT NOT NULL CHECK (event_type IN ('human', 'ai', 'cited')),
                    text_hash TEXT NOT NULL,
                    source TEXT NOT NULL,
                    span_length INTEGER NOT NULL,
                    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
                );
                CREATE INDEX idx_events_timestamp ON events(timestamp);
                CREATE INDEX idx_events_type ON events(event_type);
                CREATE INDEX idx_events_source ON events(source);
            "#,
            kind: MigrationKind::Up,
        },
        Migration {
            version: 2,
            description: "create_document_metadata_table",
            sql: r#"
                CREATE TABLE document_metadata (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    document_hash TEXT UNIQUE NOT NULL,
                    title TEXT,
                    author TEXT,
                    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                    signature TEXT,
                    public_key TEXT
                );
                CREATE INDEX idx_document_hash ON document_metadata(document_hash);
            "#,
            kind: MigrationKind::Up,
        },
    ]
}

#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// AIDEV-NOTE: Write path - all editor changes flow through this function for audit trail
#[tauri::command]
pub async fn log_provenance_event(
    app: AppHandle,
    event: ProvenanceEvent,
) -> Result<EventResponse, String> {
    use tauri_plugin_sql::{SqlitePool, Builder};
    
    // Get database connection
    let pool = app.state::<SqlitePool>();
    
    // Generate text hash for the event
    let mut hasher = Sha256::new();
    hasher.update(event.text_hash.as_bytes());
    let text_hash = format!("{:x}", hasher.finalize());
    
    // Insert event into database
    let result = sqlx::query!(
        r#"
        INSERT INTO events (timestamp, event_type, text_hash, source, span_length)
        VALUES (?1, ?2, ?3, ?4, ?5)
        "#,
        event.timestamp,
        event.event_type,
        text_hash,
        event.source,
        event.span_length as i64
    )
    .execute(&**pool)
    .await
    .map_err(|e| format!("Database insert failed: {}", e))?;
    
    Ok(EventResponse {
        id: result.last_insert_rowid(),
        text_hash,
    })
}

// AIDEV-NOTE: Read path - supports filtering by type/limit for manifest generation and UI
#[tauri::command]
pub async fn get_event_history(
    app: AppHandle,
    limit: Option<u32>,
    event_type: Option<String>,
) -> Result<Vec<ProvenanceEvent>, String> {
    let pool = app.state::<SqlitePool>();
    let limit = limit.unwrap_or(100) as i64;
    
    let events = if let Some(filter_type) = event_type {
        sqlx::query_as!(
            ProvenanceEvent,
            r#"
            SELECT timestamp, event_type, text_hash, source, span_length as "span_length: usize"
            FROM events 
            WHERE event_type = ?1
            ORDER BY timestamp DESC 
            LIMIT ?2
            "#,
            filter_type,
            limit
        )
        .fetch_all(&**pool)
        .await
        .map_err(|e| format!("Database query failed: {}", e))?
    } else {
        sqlx::query_as!(
            ProvenanceEvent,
            r#"
            SELECT timestamp, event_type, text_hash, source, span_length as "span_length: usize"
            FROM events 
            ORDER BY timestamp DESC 
            LIMIT ?1
            "#,
            limit
        )
        .fetch_all(&**pool)
        .await
        .map_err(|e| format!("Database query failed: {}", e))?
    };
    
    Ok(events)
}

// AIDEV-NOTE: Analytics engine - calculates percentages and stats for transparency reports
#[tauri::command]
pub async fn generate_manifest(app: AppHandle) -> Result<ManifestData, String> {
    let pool = app.state::<SqlitePool>();
    
    // Get provenance statistics
    let stats = sqlx::query!(
        r#"
        SELECT 
            event_type,
            SUM(span_length) as total_length,
            COUNT(*) as event_count
        FROM events 
        GROUP BY event_type
        "#
    )
    .fetch_all(&**pool)
    .await
    .map_err(|e| format!("Database query failed: {}", e))?;
    
    let mut type_stats: HashMap<String, usize> = HashMap::new();
    let mut total_characters = 0;
    
    for stat in stats {
        let length = stat.total_length.unwrap_or(0) as usize;
        type_stats.insert(stat.event_type.clone(), length);
        total_characters += length;
    }
    
    // Calculate percentages
    let human_chars = type_stats.get("human").unwrap_or(&0);
    let ai_chars = type_stats.get("ai").unwrap_or(&0);
    let cited_chars = type_stats.get("cited").unwrap_or(&0);
    
    let human_percentage = if total_characters > 0 { (*human_chars as f64 / total_characters as f64) * 100.0 } else { 0.0 };
    let ai_percentage = if total_characters > 0 { (*ai_chars as f64 / total_characters as f64) * 100.0 } else { 0.0 };
    let cited_percentage = if total_characters > 0 { (*cited_chars as f64 / total_characters as f64) * 100.0 } else { 0.0 };
    
    // Get recent events
    let events = get_event_history(app, Some(50), None).await?;
    
    Ok(ManifestData {
        human_percentage,
        ai_percentage,
        cited_percentage,
        total_characters,
        events,
    })
}

// AIDEV-NOTE: AI gateway - handles OpenAI API calls with proper error handling and attribution
#[tauri::command]
pub async fn query_ai_assistant(
    prompt_data: AIPrompt,
) -> Result<AIResponse, String> {
    let api_key = std::env::var("OPENAI_API_KEY")
        .map_err(|_| "OPENAI_API_KEY environment variable not set")?;
    
    let client = reqwest::Client::new();
    let model = prompt_data.model.unwrap_or_else(|| "gpt-3.5-turbo".to_string());
    
    let request_body = serde_json::json!({
        "model": model,
        "messages": [
            {
                "role": "user",
                "content": prompt_data.prompt
            }
        ],
        "max_tokens": prompt_data.max_tokens.unwrap_or(1000),
        "temperature": 0.7
    });
    
    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("API request failed: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("API error: {}", response.status()));
    }
    
    let response_data: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;
    
    let content = response_data["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("")
        .to_string();
    
    let token_count = response_data["usage"]["total_tokens"]
        .as_u64()
        .map(|t| t as u32);
    
    Ok(AIResponse {
        content,
        model,
        token_count,
    })
}

// AIDEV-NOTE: Security core - signs documents with ed25519 for tamper-proof verification
#[tauri::command]
pub async fn sign_document(
    content: String,
    private_key_bytes: Vec<u8>,
) -> Result<String, String> {
    use ed25519_dalek::{Signer, SigningKey};
    
    // Parse private key
    let signing_key = SigningKey::from_bytes(
        &private_key_bytes.try_into()
            .map_err(|_| "Invalid private key length")?
    );
    
    // Create signature
    let signature = signing_key.sign(content.as_bytes());
    
    // Return base64-encoded signature
    Ok(base64::encode(signature.to_bytes()))
}

// AIDEV-NOTE: Key generation - creates cryptographically secure keypairs for document identity
#[tauri::command]
pub fn generate_keypair() -> Result<(String, String), String> {
    use ed25519_dalek::{SigningKey, VerifyingKey};
    use rand::rngs::OsRng;
    
    let mut csprng = OsRng {};
    let signing_key = SigningKey::generate(&mut csprng);
    let verifying_key: VerifyingKey = signing_key.verifying_key();
    
    let private_key = base64::encode(signing_key.to_bytes());
    let public_key = base64::encode(verifying_key.to_bytes());
    
    Ok((private_key, public_key))
}

// AIDEV-NOTE: Verification endpoint - validates document authenticity using public key crypto
#[tauri::command]
pub fn verify_signature(
    content: String,
    signature_b64: String,
    public_key_b64: String,
) -> Result<bool, String> {
    use ed25519_dalek::{Verifier, VerifyingKey, Signature};
    
    let public_key_bytes = base64::decode(public_key_b64)
        .map_err(|_| "Invalid public key encoding")?;
    let signature_bytes = base64::decode(signature_b64)
        .map_err(|_| "Invalid signature encoding")?;
    
    let verifying_key = VerifyingKey::from_bytes(
        &public_key_bytes.try_into()
            .map_err(|_| "Invalid public key length")?
    ).map_err(|_| "Invalid public key format")?;
    
    let signature = Signature::from_bytes(
        &signature_bytes.try_into()
            .map_err(|_| "Invalid signature length")?
    );
    
    match verifying_key.verify(content.as_bytes(), &signature) {
        Ok(()) => Ok(true),
        Err(_) => Ok(false),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_sql::Builder::default()
                .add_migrations("sqlite:sonnun.db", create_migrations())
                .build()
        )
        .invoke_handler(tauri::generate_handler![
            greet,
            log_provenance_event,
            get_event_history,
            generate_manifest,
            query_ai_assistant,
            sign_document,
            generate_keypair,
            verify_signature
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
