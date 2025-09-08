use wasmbed_cert::ServerIdentity;
use wasmbed_tls_utils::TlsUtils;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔐 Testing Gateway TLS Integration");
    println!("==================================");
    
    // Read the files exactly like the gateway does
    let private_key_bytes = fs::read("certs/server-key-pkcs8.pem")?;
    let certificate_bytes = fs::read("certs/server-cert.pem")?;
    let client_ca_bytes = fs::read("certs/ca-cert.pem")?;
    
    println!("✅ Read private key: {} bytes", private_key_bytes.len());
    println!("✅ Read certificate: {} bytes", certificate_bytes.len());
    println!("✅ Read client CA: {} bytes", client_ca_bytes.len());
    
    // Parse using our TLS utils
    let private_key = TlsUtils::parse_private_key(&private_key_bytes)?;
    let certificate = TlsUtils::parse_certificate(&certificate_bytes)?;
    let client_ca_certs = TlsUtils::parse_certificates(&client_ca_bytes)?;
    
    println!("✅ Parsed private key");
    println!("✅ Parsed certificate");
    println!("✅ Parsed {} client CA certificates", client_ca_certs.len());
    
    // Create ServerIdentity exactly like the gateway does
    let identity = ServerIdentity::from_parts(
        match private_key {
            rustls_pki_types::PrivateKeyDer::Pkcs8(pkcs8) => pkcs8,
            _ => return Err("Only PKCS8 private keys are supported".into()),
        },
        certificate,
    );
    
    println!("✅ Created ServerIdentity successfully");
    
    let client_ca = client_ca_certs
        .into_iter()
        .next()
        .ok_or("No CA certificate found in PEM file")?;
    
    println!("✅ Extracted client CA certificate");
    
    println!("\n🎉 Gateway TLS Integration Test Complete!");
    Ok(())
}
