use rustls_pemfile;
use std::io::Cursor;

fn main() {
    let key_data = std::fs::read("server-key-pkcs8.pem").unwrap();
    let mut reader = Cursor::new(&key_data);
    
    match rustls_pemfile::pkcs8_private_keys(&mut reader).collect::<Result<Vec<_>, _>>() {
        Ok(keys) => {
            println!("Successfully parsed {} PKCS8 keys", keys.len());
            if !keys.is_empty() {
                println!("First key length: {} bytes", keys[0].secret_pkcs8_der().len());
            }
        },
        Err(e) => {
            println!("Failed to parse PKCS8 keys: {}", e);
        }
    }
}
