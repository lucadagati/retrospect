# Wasmbed Scripts

Questa directory contiene tutti gli script per gestire la piattaforma Wasmbed, organizzati per categoria.

## 📁 Struttura

### 🚀 Setup & Deployment (`setup/`)
Script per configurare e deployare la piattaforma:
- `01-setup-platform.sh` - Setup iniziale della piattaforma
- `02-deploy-to-k8s.sh` - Deploy su Kubernetes
- `03-build-images.sh` - Build delle immagini Docker
- `04-cleanup-platform.sh` - Pulizia della piattaforma
- `20-dev-environment.sh` - Setup ambiente di sviluppo

### 🧪 Testing (`testing/`)
Script per eseguire test:
- `05-test-complete.sh` - Test completo del sistema
- `06-test-unit.sh` - Test unitari
- `07-test-integration.sh` - Test di integrazione
- `08-test-end-to-end.sh` - Test end-to-end
- `09-test-security.sh` - Test di sicurezza
- `run-all-tests.sh` - Esegue tutti i test

### 🔐 Security (`security/`)
Script per gestione sicurezza:
- `10-security-scan.sh` - Scansione di sicurezza
- `11-security-hardening.sh` - Hardening di sicurezza
- `12-certificate-rotate.sh` - Rotazione certificati
- `generate-certs.sh` - Generazione certificati

### 📊 Monitoring & Maintenance (`monitoring/`)
Script per monitoraggio e manutenzione:
- `13-monitor-platform.sh` - Monitoraggio piattaforma
- `14-show-logs.sh` - Visualizzazione log
- `15-show-status.sh` - Stato del sistema
- `16-health-check.sh` - Health check
- `17-backup-platform.sh` - Backup piattaforma
- `18-restore-platform.sh` - Restore piattaforma
- `19-disaster-recovery.sh` - Disaster recovery

### 🛠️ Development (`development/`)
Script per sviluppo:
- `21-build-all.sh` - Build completo
- `22-lint-code.sh` - Linting del codice
- `23-generate-docs.sh` - Generazione documentazione

## 🚀 Utilizzo Rapido

### Setup Iniziale
```bash
./scripts/setup/01-setup-platform.sh
```

### Test Completo
```bash
./scripts/testing/run-all-tests.sh
```

### Deploy su Kubernetes
```bash
./scripts/setup/02-deploy-to-k8s.sh
```

### Monitoraggio
```bash
./scripts/monitoring/13-monitor-platform.sh
```

## 📋 Prerequisiti

- Rust 1.88+
- Docker
- kubectl
- k3d (per sviluppo locale)
- Nix (opzionale, per ambiente di sviluppo)

## 🔧 Configurazione

Gli script utilizzano variabili d'ambiente per la configurazione. Vedi `docs/development/setup.md` per dettagli.
