// Simple test runner for core functionality without Tauri dependencies
use sonnun_lib::crypto_utils::{hash_text, generate_keypair, sign_document, verify_signature};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_text() {
        let text = "Hello, world!";
        let hash = hash_text(text);
        assert_eq!(hash.len(), 64);
        assert_eq!(hash, hash_text(text));
    }

    #[test]
    fn test_generate_keypair() {
        let result = generate_keypair();
        assert!(result.is_ok());
        let (private_key, public_key) = result.unwrap();
        assert!(!private_key.is_empty());
        assert!(!public_key.is_empty());
        assert!(base64::decode(&private_key).is_ok());
        assert!(base64::decode(&public_key).is_ok());
    }

    #[tokio::test]
    async fn test_sign_and_verify_document() {
        let content = "This is a test document.";
        let (private_key_b64, public_key_b64) = generate_keypair().unwrap();
        let private_key_bytes = base64::decode(&private_key_b64).unwrap();
        let signature = sign_document(content.to_string(), private_key_bytes).await.unwrap();
        let result = verify_signature(content.to_string(), signature, public_key_b64).unwrap();
        assert!(result);
    }

    #[tokio::test]
    async fn test_sign_document_validation() {
        let result = sign_document("".to_string(), vec![0; 32]).await;
        assert!(result.is_err());
        let result = sign_document("test".to_string(), vec![0; 10]).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_signature_validation() {
        let result = verify_signature("".to_string(), "sig".to_string(), "key".to_string());
        assert!(result.is_err());
        let result = verify_signature("test".to_string(), "invalid_base64!".to_string(), "key".to_string());
        assert!(result.is_err());
    }
}

#[tokio::main]
async fn main() {
    println!("Running core functionality tests...");
    let text = "Hello, world!";
    let hash = hash_text(text);
    println!("✓ Hash length: {} (expected: 64)", hash.len());
    println!("✓ Hash consistency: {}", hash == hash_text(text));

    println!("Testing crypto functions...");
    match generate_keypair() {
        Ok((private_key, public_key)) => {
            println!("✓ Key generation successful");
            println!("✓ Private key length: {}", private_key.len());
            println!("✓ Public key length: {}", public_key.len());

            if let Ok(private_key_bytes) = base64::decode(&private_key) {
                if let Ok(signature) = sign_document("Test document content".to_string(), private_key_bytes).await {
                    if let Ok(is_valid) = verify_signature("Test document content".to_string(), signature, public_key) {
                        println!("✓ Signature verification: {}", is_valid);
                    }
                }
            }
        }
        Err(e) => println!("✗ Key generation failed: {}", e),
    }

    println!("Core functionality tests completed!");
}
