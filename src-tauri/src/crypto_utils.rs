use sha2::{Digest, Sha256};

/// Generate a SHA256 hex digest for the provided text.
pub fn hash_text(text: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Sign the given document contents using the provided private key bytes.
#[tauri::command]
pub async fn sign_document(
    content: String,
    private_key_bytes: Vec<u8>,
) -> Result<String, String> {
    use ed25519_dalek::{Signer, SigningKey};

    if content.is_empty() {
        return Err("Content cannot be empty".to_string());
    }

    let signing_key = SigningKey::from_bytes(
        &private_key_bytes
            .try_into()
            .map_err(|_| "Invalid private key length")?,
    );

    let signature = signing_key.sign(content.as_bytes());
    Ok(base64::encode(signature.to_bytes()))
}

/// Generate an ed25519 key pair returned as base64 encoded strings.
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

/// Verify the signature for the given document using the provided public key.
#[tauri::command]
pub fn verify_signature(
    content: String,
    signature_b64: String,
    public_key_b64: String,
) -> Result<bool, String> {
    use ed25519_dalek::{Signature, Verifier, VerifyingKey};

    if content.is_empty() {
        return Err("Content cannot be empty".to_string());
    }

    let public_key_bytes = base64::decode(public_key_b64)
        .map_err(|_| "Invalid public key encoding")?;
    let signature_bytes = base64::decode(signature_b64)
        .map_err(|_| "Invalid signature encoding")?;

    let verifying_key = VerifyingKey::from_bytes(
        &public_key_bytes
            .try_into()
            .map_err(|_| "Invalid public key length")?,
    )
    .map_err(|_| "Invalid public key format")?;

    let signature = Signature::from_bytes(
        &signature_bytes
            .try_into()
            .map_err(|_| "Invalid signature length")?,
    );

    match verifying_key.verify(content.as_bytes(), &signature) {
        Ok(()) => Ok(true),
        Err(_) => Ok(false),
    }
}

