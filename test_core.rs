// Simple test runner for core functionality without Tauri dependencies
use std::fs;

// Copy the core modules we want to test
mod hash_text_test {
    use sha2::{Sha256, Digest};
    
    pub fn hash_text(text: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(text.as_bytes());
        format!("{:x}", hasher.finalize())
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
    }
}

mod crypto_test {
    use ed25519_dalek::{Signer, Verifier, SigningKey, VerifyingKey, Signature};
    use rand::rngs::OsRng;
    
    pub fn generate_keypair() -> Result<(String, String), String> {
        let mut csprng = OsRng {};
        let signing_key = SigningKey::generate(&mut csprng);
        let verifying_key: VerifyingKey = signing_key.verifying_key();
        
        let private_key = base64::encode(signing_key.to_bytes());
        let public_key = base64::encode(verifying_key.to_bytes());
        
        Ok((private_key, public_key))
    }
    
    pub fn sign_document(content: String, private_key_bytes: Vec<u8>) -> Result<String, String> {
        if content.is_empty() {
            return Err("Content cannot be empty".to_string());
        }
        
        let signing_key = SigningKey::from_bytes(
            &private_key_bytes.try_into()
                .map_err(|_| "Invalid private key length")?
        );
        
        let signature = signing_key.sign(content.as_bytes());
        Ok(base64::encode(signature.to_bytes()))
    }
    
    pub fn verify_signature(
        content: String,
        signature_b64: String,
        public_key_b64: String,
    ) -> Result<bool, String> {
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
    
    #[cfg(test)]
    mod tests {
        use super::*;
        
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
            
            let signature_result = sign_document(content.to_string(), private_key_bytes);
            assert!(signature_result.is_ok());
            
            let signature = signature_result.unwrap();
            
            let verification_result = verify_signature(
                content.to_string(),
                signature,
                public_key_b64
            );
            assert!(verification_result.is_ok());
            assert!(verification_result.unwrap());
        }
        
        #[test]
        fn test_sign_document_validation() {
            // Test empty content
            let result = sign_document("".to_string(), vec![0; 32]);
            assert!(result.is_err());
            assert!(result.unwrap_err().contains("Content cannot be empty"));
            
            // Test invalid key length
            let result = sign_document("test".to_string(), vec![0; 10]);
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
    }
}

fn main() {
    println!("Running core functionality tests...");
    
    // Run hash tests
    println!("Testing hash function...");
    let text = "Hello, world!";
    let hash = hash_text_test::hash_text(text);
    println!("✓ Hash length: {} (expected: 64)", hash.len());
    println!("✓ Hash consistency: {}", hash == hash_text_test::hash_text(text));
    
    // Run crypto tests
    println!("Testing crypto functions...");
    match crypto_test::generate_keypair() {
        Ok((private_key, public_key)) => {
            println!("✓ Key generation successful");
            println!("✓ Private key length: {}", private_key.len());
            println!("✓ Public key length: {}", public_key.len());
            
            // Test signing and verification
            let content = "Test document content";
            if let Ok(private_key_bytes) = base64::decode(&private_key) {
                if let Ok(signature) = crypto_test::sign_document(content.to_string(), private_key_bytes) {
                    println!("✓ Document signing successful");
                    
                    if let Ok(is_valid) = crypto_test::verify_signature(
                        content.to_string(),
                        signature,
                        public_key
                    ) {
                        println!("✓ Signature verification: {}", is_valid);
                    }
                }
            }
        },
        Err(e) => println!("✗ Key generation failed: {}", e),
    }
    
    println!("Core functionality tests completed!");
}