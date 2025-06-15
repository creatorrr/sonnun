use clap::{Arg, Command};
use std::fs;
use serde_json::Value;
use ed25519_dalek::{PublicKey, Signature, Verifier};
use base64::{engine::general_purpose, Engine as _};

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

    let manifest_start = content
        .find(r#"<script type=\"application/json\" id=\"sonnun-manifest\">"#)
        .ok_or("No Sonnun manifest found in document")?;
    let manifest_content_start = content[manifest_start..]
        .find('>')
        .ok_or("Malformed manifest script tag")?;
    let manifest_content_end = content[manifest_start..]
        .find("</script>")
        .ok_or("Manifest script tag not closed")?;

    let manifest_json = &content[manifest_start + manifest_content_start + 1..manifest_start + manifest_content_end];
    let signed_manifest: Value = serde_json::from_str(manifest_json.trim())
        .map_err(|e| format!("Invalid manifest JSON: {}", e))?;

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

    let public_key_bytes = general_purpose::STANDARD
        .decode(public_key_b64)
        .map_err(|e| format!("Invalid public key encoding: {}", e))?;
    let signature_bytes = general_purpose::STANDARD
        .decode(signature_b64)
        .map_err(|e| format!("Invalid signature encoding: {}", e))?;

    let public_key = PublicKey::from_bytes(&public_key_bytes)
        .map_err(|e| format!("Invalid public key: {}", e))?;
    let signature = Signature::from_bytes(&signature_bytes)
        .map_err(|e| format!("Invalid signature: {}", e))?;

    let canonical_manifest = serde_json::to_string(&manifest)
        .map_err(|e| format!("Failed to serialize manifest: {}", e))?;

    let valid = public_key
        .verify(canonical_manifest.as_bytes(), &signature)
        .is_ok();

    Ok(VerificationResult {
        valid,
        public_key: public_key_b64.to_string(),
        manifest,
    })
}

