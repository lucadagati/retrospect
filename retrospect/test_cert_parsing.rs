use std::fs;
use wasmbed_tls_utils::TlsUtils;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing certificate parsing...");
    
    // Test CA certificate parsing
    let ca_bytes = fs::read("certs/ca-cert.pem")?;
    println!("CA cert size: {} bytes", ca_bytes.len());
    
    match TlsUtils::parse_certificates(&ca_bytes) {
        Ok(certs) => {
            println!("✅ CA certificates parsed successfully: {} certificates", certs.len());
            for (i, cert) in certs.iter().enumerate() {
                println!("  Certificate {}: {} bytes", i, cert.len());
            }
        }
        Err(e) => {
            println!("❌ CA certificate parsing failed: {:?}", e);
            return Err(e.into());
        }
    }
    
    // Test gateway certificate parsing
    let gateway_bytes = fs::read("certs/gateway-cert.pem")?;
    println!("Gateway cert size: {} bytes", gateway_bytes.len());
    
    match TlsUtils::parse_certificate(&gateway_bytes) {
        Ok(cert) => {
            println!("✅ Gateway certificate parsed successfully: {} bytes", cert.len());
        }
        Err(e) => {
            println!("❌ Gateway certificate parsing failed: {:?}", e);
            return Err(e.into());
        }
    }
    
    println!("All certificate parsing tests passed!");
    Ok(())
}
