use clap::{Arg, Command};
use std::fs;
use serde_json::Value;
use ed25519_dalek::{VerifyingKey, Signature, Verifier};
use regex::Regex;

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

// AIDEV-NOTE: Document verification core - parses HTML, validates manifest, verifies crypto signature
fn verify_document(filename: &str, provided_key: Option<&String>) -> Result<VerificationResult, String> {
    let content = fs::read_to_string(filename).map_err(|e| format!("Failed to read file: {}", e))?;

    // AIDEV-NOTE: Robust HTML parsing - uses regex to handle whitespace/formatting variations
    let manifest_regex = Regex::new(r#"<script[^>]*type="application/json"[^>]*id="sonnun-manifest"[^>]*>([\s\S]*?)</script>"#)
        .map_err(|e| format!("Regex compilation error: {}", e))?;
    
    let captures = manifest_regex.captures(&content)
        .ok_or("No Sonnun manifest found in document")?;
    
    let manifest_json = captures.get(1)
        .ok_or("Failed to extract manifest content")?
        .as_str().trim();
    
    let signed_manifest: Value = serde_json::from_str(manifest_json)
        .map_err(|e| format!("Invalid manifest JSON: {}", e))?;

    // AIDEV-NOTE: Manifest validation - ensures required fields exist before access
    if !signed_manifest.is_object() {
        return Err("Manifest must be a JSON object".to_string());
    }
    
    let manifest = signed_manifest.get("manifest")
        .ok_or("Missing 'manifest' field in signed manifest")?
        .clone();
    
    let signature_b64 = signed_manifest.get("signature")
        .and_then(|v| v.as_str())
        .ok_or("Missing or invalid 'signature' field in manifest")?;
    
    let public_key_b64 = signed_manifest.get("public_key")
        .and_then(|v| v.as_str())
        .ok_or("Missing or invalid 'public_key' field in manifest")?;

    if let Some(key) = provided_key {
        if key != public_key_b64 {
            return Err("Provided public key does not match document key".to_string());
        }
    }

    // AIDEV-NOTE: Consistent crypto API - matches main app patterns (src-tauri/src/lib.rs:211,216)
    let public_key_bytes = base64::decode(public_key_b64)
        .map_err(|e| format!("Invalid public key encoding: {}", e))?;
    let signature_bytes = base64::decode(signature_b64)
        .map_err(|e| format!("Invalid signature encoding: {}", e))?;

    let verifying_key = VerifyingKey::from_bytes(
        &public_key_bytes.try_into()
            .map_err(|_| "Invalid public key length")?)
        .map_err(|e| format!("Invalid public key format: {}", e))?;
    
    let signature = Signature::from_bytes(
        &signature_bytes.try_into()
            .map_err(|_| "Invalid signature length")?)
        .map_err(|e| format!("Invalid signature format: {}", e))?;

    let canonical_manifest = serde_json::to_string(&manifest)
        .map_err(|e| format!("Failed to serialize manifest: {}", e))?;

    let valid = verifying_key
        .verify(canonical_manifest.as_bytes(), &signature)
        .is_ok();

    Ok(VerificationResult {
        valid,
        public_key: public_key_b64.to_string(),
        manifest,
    })
}

