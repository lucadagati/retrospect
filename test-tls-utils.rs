use wasmbed_tls_utils::TlsUtils;
use std::fs;

fn main() {
    println!("🔐 Testing Wasmbed TLS Utils Library");
    println!("====================================");
    
    // Test with PKCS8 key
    match fs::read("test-key-pkcs8.pem") {
        Ok(key_data) => {
            println!("✅ Successfully read PKCS8 key file, {} bytes", key_data.len());
            match TlsUtils::parse_private_key(&key_data) {
                Ok(_) => println!("✅ Successfully parsed PKCS8 private key"),
                Err(e) => println!("❌ Failed to parse PKCS8 private key: {}", e),
            }
        },
        Err(e) => println!("❌ Failed to read PKCS8 key file: {}", e),
    }
    
    // Test with certificate
    match fs::read("test-cert.pem") {
        Ok(cert_data) => {
            println!("✅ Successfully read certificate file, {} bytes", cert_data.len());
            match TlsUtils::parse_certificate(&cert_data) {
                Ok(_) => println!("✅ Successfully parsed certificate"),
                Err(e) => println!("❌ Failed to parse certificate: {}", e),
            }
        },
        Err(e) => println!("❌ Failed to read certificate file: {}", e),
    }
    
    println!("\n🎉 TLS Utils Library Test Complete!");
}
