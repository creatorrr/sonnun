// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tauri_plugin_sql::{Builder as SqlBuilder, Migration, MigrationKind};

mod database;
use database::Database;

// AIDEV-NOTE: Foundation types - these structs define the entire provenance data model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvenanceEvent {
    pub timestamp: String,
    pub event_type: String,  // "human", "ai", "cited"
    /// Plain text used for hashing prior to storage
    pub text: String,
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

const DB_URL: &str = "sqlite:sonnun.db";

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

// AIDEV-NOTE: Security improvement - proper hash generation for provenance events
pub fn hash_text(text: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// AIDEV-NOTE: Write path - all editor changes flow through this function for audit trail
#[tauri::command]
pub async fn log_provenance_event(
    event: ProvenanceEvent,
) -> Result<EventResponse, String> {
    let db = Database::new(DB_URL).await?;
    let mut event_with_hash = event.clone();
    
    // Generate proper text hash 
    event_with_hash.text_hash = hash_text(&event.text_hash);
    
    db.insert_event(event_with_hash).await
}

// AIDEV-NOTE: Read path - supports filtering by type/limit for manifest generation and UI
#[tauri::command]
pub async fn get_event_history(
    limit: Option<u32>,
    event_type: Option<String>,
) -> Result<Vec<ProvenanceEvent>, String> {
    let db = Database::new(DB_URL).await?;
    db.get_events(limit, event_type).await
}

// AIDEV-NOTE: Analytics engine - calculates percentages and stats for transparency reports
#[tauri::command]
pub async fn generate_manifest() -> Result<ManifestData, String> {
    let db = Database::new(DB_URL).await?;
    db.generate_manifest().await
}

// AIDEV-NOTE: AI gateway - handles OpenAI API calls with proper error handling and attribution
#[tauri::command]
pub async fn query_ai_assistant(
    prompt_data: AIPrompt,
) -> Result<AIResponse, String> {
    let api_key = std::env::var("OPENAI_API_KEY")
        .map_err(|_| "OPENAI_API_KEY environment variable not set")?;
    
    if prompt_data.prompt.trim().is_empty() {
        return Err("Prompt cannot be empty".to_string());
    }
    
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
        let status = response.status();
        let error_text = response.text().await
            .unwrap_or_else(|_| "Failed to read error response".to_string());
        return Err(format!("API error {}: {}", status, error_text));
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
    
    if content.is_empty() {
        return Err("Content cannot be empty".to_string());
    }
    
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
    
    if content.is_empty() {
        return Err("Content cannot be empty".to_string());
    }
    
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

fn create_migrations() -> Vec<Migration> {
    vec![Migration {
        version: 1,
        description: "create events table",
        sql: "CREATE TABLE IF NOT EXISTS events (\n  id INTEGER PRIMARY KEY AUTOINCREMENT,\n  timestamp TEXT NOT NULL,\n  event_type TEXT NOT NULL,\n  text_hash TEXT NOT NULL,\n  source TEXT,\n  span_length INTEGER\n);",
        kind: MigrationKind::Up,
    }]
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(
            SqlBuilder::default()
                .add_migrations("sqlite:sonnun.db", create_migrations())
                .build(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_text() {
        let text = "Hello, world!";
        let hash = hash_text(text);
        assert_eq!(hash.len(), 64); // SHA256 produces 32-byte (64 hex char) hash
        assert_eq!(hash, hash_text(text)); // Same input produces same hash
    }

    #[test]
    fn test_generate_keypair() {
        let result = generate_keypair();
        assert!(result.is_ok());
        
        let (private_key, public_key) = result.unwrap();
        assert!(!private_key.is_empty());
        assert!(!public_key.is_empty());
        
        // Check base64 encoding validity
        assert!(base64::decode(&private_key).is_ok());
        assert!(base64::decode(&public_key).is_ok());
    }

    #[tokio::test]
    async fn test_sign_and_verify_document() {
        let content = "This is a test document.";
        let (private_key_b64, public_key_b64) = generate_keypair().unwrap();
        let private_key_bytes = base64::decode(&private_key_b64).unwrap();
        
        // Test signing
        let signature_result = sign_document(content.to_string(), private_key_bytes).await;
        assert!(signature_result.is_ok());
        
        let signature = signature_result.unwrap();
        
        // Test verification
        let verification_result = verify_signature(
            content.to_string(),
            signature,
            public_key_b64
        );
        assert!(verification_result.is_ok());
        assert!(verification_result.unwrap());
    }

    #[tokio::test]
    async fn test_sign_document_validation() {
        // Test empty content
        let result = sign_document("".to_string(), vec![0; 32]).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Content cannot be empty"));
        
        // Test invalid key length
        let result = sign_document("test".to_string(), vec![0; 10]).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid private key length"));
    }

    #[test]
    fn test_verify_signature_validation() {
        // Test empty content
        let result = verify_signature("".to_string(), "sig".to_string(), "key".to_string());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Content cannot be empty"));
        
        // Test invalid base64
        let result = verify_signature("test".to_string(), "invalid_base64!".to_string(), "key".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_provenance_event_serialization() {
        let event = ProvenanceEvent {
            timestamp: "2023-01-01T00:00:00Z".to_string(),
            event_type: "human".to_string(),
            text: "test_hash".to_string(),
            source: "user".to_string(),
            span_length: 10,
        };
        
        let json = serde_json::to_string(&event);
        assert!(json.is_ok());
        
        let deserialized: Result<ProvenanceEvent, _> = serde_json::from_str(&json.unwrap());
        assert!(deserialized.is_ok());
        
        let deserialized_event = deserialized.unwrap();
        assert_eq!(event.timestamp, deserialized_event.timestamp);
        assert_eq!(event.event_type, deserialized_event.event_type);
    }
}