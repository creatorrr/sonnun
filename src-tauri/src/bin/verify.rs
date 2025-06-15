use clap::{Arg, Command};
use std::fs;
use serde_json::Value;
use ed25519_dalek::{VerifyingKey, Signature, Verifier};
use base64::{engine::general_purpose, Engine as _};
use regex::Regex;

// AIDEV-NOTE: CLI verifier for Sonnun signed documents - validates ed25519 signatures

fn main() {
    let matches = Command::new("sonnun-verify")
        .version("1.0")
        .about("Verify Sonnun signed documents")
        .arg(
            Arg::new("file")
                .help("HTML file to verify")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::new("public-key")
                .short('k')
                .long("key")
                .value_name("KEY")
                .help("Public key to verify against (base64)"),
        )
        .get_matches();

    let filename = matches.get_one::<String>("file").unwrap();
    let provided_key = matches.get_one::<String>("public-key");

    match verify_document(filename, provided_key) {
        Ok(result) => {
            if result.valid {
                println!("✅ VALID signature");
                println!("Public key: {}", result.public_key);
                println!("Manifest: {}", serde_json::to_string_pretty(&result.manifest).unwrap());
            } else {
                println!("❌ INVALID signature");
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

struct VerificationResult {
    valid: bool,
    public_key: String,
    manifest: Value,
}

fn verify_document(filename: &str, provided_key: Option<&String>) -> Result<VerificationResult, String> {
    let content = fs::read_to_string(filename).map_err(|e| format!("Failed to read file: {}", e))?;

    // AIDEV-NOTE: Regex-based HTML parsing for manifest extraction - more robust than string matching
    let manifest_regex = Regex::new(
        r#"<script\s+type\s*=\s*["']application/json["']\s+id\s*=\s*["']sonnun-manifest["']\s*>(.*?)</script>"#
    ).map_err(|e| format!("Failed to compile regex: {}", e))?;
    
    let captures = manifest_regex.captures(&content)
        .ok_or("No Sonnun manifest found in document")?;
    
    let manifest_json = captures.get(1)
        .ok_or("Failed to extract manifest content")?
        .as_str();
    let signed_manifest: Value = serde_json::from_str(manifest_json.trim())
        .map_err(|e| format!("Invalid manifest JSON: {}", e))?;

    // AIDEV-NOTE: Manifest validation - ensure required fields exist before accessing
    validate_manifest_structure(&signed_manifest)?;
    
    let manifest = signed_manifest["manifest"].clone();
    let signature_b64 = signed_manifest["signature"]
        .as_str()
        .ok_or("No signature in manifest")?;
    let public_key_b64 = signed_manifest["public_key"]
        .as_str()
        .ok_or("No public key in manifest")?;

    if let Some(key) = provided_key {
        if key != public_key_b64 {
            return Err("Provided public key does not match document key".to_string());
        }
    }

    // AIDEV-NOTE: Base64 decode - using general_purpose::STANDARD engine per base64 v0.21 API
    let public_key_bytes = general_purpose::STANDARD
        .decode(public_key_b64)
        .map_err(|e| format!("Invalid public key encoding: {}", e))?;
    let signature_bytes = general_purpose::STANDARD
        .decode(signature_b64)
        .map_err(|e| format!("Invalid signature encoding: {}", e))?;

    // AIDEV-NOTE: Ed25519 key construction - requires exactly 32 bytes via array reference
    let verifying_key = VerifyingKey::from_bytes(
        &public_key_bytes.try_into()
            .map_err(|_| "Invalid public key length")?
    ).map_err(|e| format!("Invalid public key: {}", e))?;
    
    // AIDEV-NOTE: Ed25519 signature - requires exactly 64 bytes via array reference
    let signature = Signature::from_bytes(
        &signature_bytes.try_into()
            .map_err(|_| "Invalid signature length")?
    );

    // AIDEV-NOTE: Canonical JSON serialization ensures consistent signature verification
    let canonical_manifest = serde_json::to_string(&manifest)
        .map_err(|e| format!("Failed to serialize manifest: {}", e))?;

    // AIDEV-NOTE: Cryptographic verification - returns Ok() on valid signature, Err on invalid
    let valid = verifying_key
        .verify(canonical_manifest.as_bytes(), &signature)
        .is_ok();

    Ok(VerificationResult {
        valid,
        public_key: public_key_b64.to_string(),
        manifest,
    })
}

// AIDEV-NOTE: Validates manifest has required fields: manifest, signature, public_key
fn validate_manifest_structure(signed_manifest: &Value) -> Result<(), String> {
    if !signed_manifest.is_object() {
        return Err("Manifest must be a JSON object".to_string());
    }
    
    if !signed_manifest.get("manifest").is_some() {
        return Err("Missing 'manifest' field in signed document".to_string());
    }
    
    if !signed_manifest.get("signature").is_some() {
        return Err("Missing 'signature' field in signed document".to_string());
    }
    
    if !signed_manifest.get("public_key").is_some() {
        return Err("Missing 'public_key' field in signed document".to_string());
    }
    
    // AIDEV-NOTE: Validate inner manifest structure for provenance tracking
    let manifest = &signed_manifest["manifest"];
    if !manifest.is_object() {
        return Err("Inner manifest must be a JSON object".to_string());
    }
    
    // Check for expected provenance fields
    let expected_fields = ["total_characters", "human_characters", "ai_characters", "cited_characters"];
    for field in expected_fields {
        if !manifest.get(field).is_some() {
            return Err(format!("Missing '{}' field in manifest", field));
        }
    }
    
    Ok(())
}

