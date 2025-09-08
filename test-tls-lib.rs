use wasmbed_tls_utils::TlsUtils;
use std::fs;

fn main() {
    // Test with our actual certificate files
    match fs::read("server-key-pkcs8.pem") {
        Ok(key_data) => {
            println!("Successfully read key file, {} bytes", key_data.len());
            match TlsUtils::parse_private_key(&key_data) {
                Ok(_) => println!("✅ Successfully parsed private key"),
                Err(e) => println!("❌ Failed to parse private key: {}", e),
            }
        },
        Err(e) => println!("❌ Failed to read key file: {}", e),
    }
    
    match fs::read("server-cert.pem") {
        Ok(cert_data) => {
            println!("Successfully read cert file, {} bytes", cert_data.len());
            match TlsUtils::parse_certificate(&cert_data) {
                Ok(_) => println!("✅ Successfully parsed certificate"),
                Err(e) => println!("❌ Failed to parse certificate: {}", e),
            }
        },
        Err(e) => println!("❌ Failed to read cert file: {}", e),
    }
}
