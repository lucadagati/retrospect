use std::fs;

fn main() {
    let ca_bytes = fs::read("certs/ca-cert.pem").expect("Failed to read CA cert");
    
    println!("CA cert size: {} bytes", ca_bytes.len());
    println!("First 50 bytes: {:?}", &ca_bytes[..50.min(ca_bytes.len())]);
    
    // Try to parse with pem crate
    match pem::parse(&ca_bytes) {
        Ok(pem_cert) => {
            println!("✅ PEM parsed successfully");
            println!("Tag: {}", pem_cert.tag());
            println!("Contents size: {} bytes", pem_cert.contents().len());
        }
        Err(e) => {
            println!("❌ PEM parsing failed: {:?}", e);
        }
    }
}
