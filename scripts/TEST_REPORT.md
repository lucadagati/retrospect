# Test e verifica piattaforma Wasmbed

## Verifica attuale

- **Build e unit test**: `cargo build --workspace && cargo test --workspace`
- **Verifica TLS e deploy**: `./scripts/verify-tls-and-deploy.sh [API_BASE_URL] [GATEWAY_HTTP_URL]`
- **Checklist dettagliate**: vedi `doc/RENODE_TLS_DEPLOY_VERIFICATION.md`, `doc/DASHBOARD_API_K8S_VERIFICATION.md`, `doc/DEVELOPMENT_STATUS.md`

## Report storici

- **API_TEST_REPORT.md**: report passato sui test delle API del dashboard (riferimento; per risultati aggiornati eseguire i test sopra).
