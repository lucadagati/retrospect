# Wasmbed TLS Utils

Una libreria Rust personalizzata per la gestione di certificati TLS e chiavi crittografiche nel progetto Wasmbed.

## Caratteristiche

- **Parsing di certificati PEM**: Supporta certificati X.509 in formato PEM
- **Parsing di chiavi private**: Supporta chiavi PKCS8 e RSA in formato PEM
- **Estrazione informazioni certificati**: Ottieni dettagli come subject, issuer, validità, ecc.
- **Validazione certificati**: Controlla scadenza e validità hostname
- **Supporto catene di certificati**: Parsing di bundle di certificati CA
- **Validazione chiave-certificato**: Verifica corrispondenza tra chiave privata e certificato

## Utilizzo

### Parsing di certificati

```rust
use wasmbed_tls_utils::TlsUtils;

let cert_pem = b"-----BEGIN CERTIFICATE-----
MIIBkTCB+wIJAKpC1HNuZliXxzTxCLaKpCCMgDrD9VARogdeJhtOoQ/hsk66u9m8i
7hD69nd6IzmTLTfQJFyp1EHGuOW2qyks2o0IwQDAOgBtFQ8BAf8EABMAqQwDgYIKo
ZIhvcNAQELBQADgYEAWVTk8aWmHAig1voP5rpJS8fRBRI0G6SWvxG5MPcymt+CvA
YF7yXpmHoluHsRUoqg9xrqqyOHrmmmSKuKfah2Q=
-----END CERTIFICATE-----";

let cert = TlsUtils::parse_certificate(cert_pem)?;
```

### Parsing di chiavi private

```rust
let key_pem = b"-----BEGIN PRIVATE KEY-----
MIIEvQIBADANBgkqhkiG9w0BAQEFAASCBKcwggSjAgEAAoIBAQCqQtTzrZZlXxzTx
CLaKpCCMgDrD9VARogdeJhtOoQ/hsk66u9m8i7hD69nd6IzmTLTfQJFyp1EHGuOW
2qyks2o0IwQDAOgBtFQ8BAf8EABMAqQwDgYIKoZIhvcNAQELBQADgYEAWVTk8aWm
HAig1voP5rpJS8fRBRI0G6SWvxG5MPcymt+CvA
YF7yXpmHoluHsRUoqg9xrqqyOHrmmmSKuKfah2Q=
-----END PRIVATE KEY-----";

let private_key = TlsUtils::parse_private_key(key_pem)?;
```

### Estrazione informazioni certificato

```rust
let info = TlsUtils::get_certificate_info(&cert)?;
println!("Subject: {}", info.subject);
println!("Issuer: {}", info.issuer);
println!("Serial Number: {}", info.serial_number);
println!("Valid From: {:?}", info.not_before);
println!("Valid Until: {:?}", info.not_after);
```

### Controllo scadenza certificato

```rust
let is_expired = TlsUtils::is_certificate_expired(&cert)?;
if is_expired {
    println!("⚠ Certificate is expired");
} else {
    println!("✓ Certificate is still valid");
}
```

### Validazione hostname

```rust
let is_valid = TlsUtils::is_certificate_valid_for_hostname(&cert, "example.com")?;
if is_valid {
    println!("✓ Certificate is valid for example.com");
} else {
    println!("✗ Certificate is not valid for example.com");
}
```

### Parsing di più certificati

```rust
let certs_pem = b"-----BEGIN CERTIFICATE-----
...cert1...
-----END CERTIFICATE-----
-----BEGIN CERTIFICATE-----
...cert2...
-----END CERTIFICATE-----";

let certificates = TlsUtils::parse_certificates(certs_pem)?;
println!("Parsed {} certificates", certificates.len());
```

## Funzionalità in Sviluppo

Le seguenti funzionalità sono pianificate ma non ancora implementate:

- **Generazione chiavi Ed25519**: Creazione di nuove coppie di chiavi
- **Creazione certificati self-signed**: Generazione di certificati per sviluppo/test
- **Configurazione TLS completa**: Creazione di configurazioni server/client rustls
- **Validazione avanzata catene**: Verifica completa delle catene di certificati

## Dipendenze

- `anyhow`: Gestione errori
- `rustls-pki-types`: Tipi per chiavi e certificati
- `rustls`: Libreria TLS
- `pem`: Parsing formato PEM
- `x509-parser`: Parsing certificati X.509
- `x509-cert`: Generazione certificati (feature "builder")
- `ed25519-dalek`: Operazioni crittografiche Ed25519
- `rand`: Generazione numeri casuali
- `der`: Codifica/decodifica DER

## Test

```bash
cargo test -p wasmbed-tls-utils
```

## Esempio Completo

Vedi `examples/tls-utils-example.rs` per un esempio completo di utilizzo.

## Note di Sicurezza

⚠️ **IMPORTANTE**: Questa libreria è in fase di sviluppo e non dovrebbe essere utilizzata in produzione senza una revisione completa della sicurezza.

- La validazione delle chiavi è semplificata per scopi di sviluppo
- La generazione di certificati non è ancora implementata
- Alcune funzioni di validazione potrebbero non essere complete

## Contributi

I contributi sono benvenuti! Per favore:

1. Apri una issue per discutere le modifiche
2. Crea un fork del repository
3. Implementa le modifiche con test appropriati
4. Invia una pull request

## Licenza

AGPL-3.0 - Vedi il file LICENSE per i dettagli.
