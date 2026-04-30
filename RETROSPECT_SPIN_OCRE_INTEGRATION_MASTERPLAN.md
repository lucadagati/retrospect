# RETROSPECT - Piano Dettagliato Integrazione SPIN + OCRE

## 1) Scopo del documento

Questo documento formalizza in modo completo:

1. Cosa e stato gia realizzato nel progetto Retrospect (con evidenze tecniche emerse dalle attivita svolte).
2. Cosa resta da integrare/rafforzare per ottenere una vera novelty scientifica e ingegneristica.
3. Come realizzare l'integrazione SPIN + OCRE in modo incrementale, verificabile e pubblicabile.

L'obiettivo finale e costruire una piattaforma unificata in cui lo stesso control-plane Kubernetes (Retrospect) orchestri in modo coerente:

- workload WASM cloud/fog su runtime SPIN;
- workload WASM edge su runtime OCRE/WAMR (Zephyr);
- stato operativo end-to-end coerente tra runtime reale e CRD Kubernetes.

---

## 2) Executive summary

### 2.1 Stato attuale (alto livello)

La base tecnologica non e da zero: il progetto ha gia validato i blocchi critici piu rischiosi.

- Lato Edge (OCRE/WAMR + Zephyr): handshake TLS, enrollment CBOR, deploy path, ack e status periodico risultano implementati e testati.
- Lato Cloud/Fog (SPIN): pipeline di packaging/deploy WASM su Kubernetes e gia disponibile in forma prototipale.
- Lato Control-plane (Retrospect): esiste la struttura Gateway/API/CRD utile ad allineare desired state e reported state.

### 2.2 Messaggio chiave

Il lavoro gia fatto serve in modo diretto alla tesi: non e una serie di test scollegati, ma una baseline concreta per la novelty "single control-plane, dual runtime-plane".

---

## 3) Stato dell'arte implementato (dettaglio)

## 3.1 Componenti gia operativi in Retrospect

### 3.1.1 Sicurezza e onboarding device

- Handshake TLS firmware-gateway portato in stato funzionante.
- Fix sulla gestione certificati/cipher suite per vincoli memoria embedded.
- Gestione connessioni anonime pre-enrollment (assenza certificato client TLS prima identificazione CBOR).
- Sequenza enrollment completata:
  - EnrollmentRequest
  - PublicKey
  - DeviceUuid
  - EnrollmentCompleted

### 3.1.2 Protocollo device-gateway

- Framing robusto dei messaggi (accumulo header + payload, no read "parziale" non gestita).
- Introduzione di meccanismi di ricezione robusta (loop/poll/timeout) per evitare perdita messaggi grandi (es. deploy).
- Heartbeat periodico e aggiornamento stato device in Kubernetes.

### 3.1.3 Deploy e lifecycle WASM lato edge

- Ricezione modulo WASM via canale TLS/CBOR.
- Caricamento e istanziazione runtime WAMR lato firmware.
- Chiamata esplicita entry-point (es. `run`) nel flusso di deploy reale.
- Emissione `DeployAck` e `ApplicationStatus` periodico lato firmware.

### 3.1.4 Coerenza stato Kubernetes

- Aggiornamento `Application CRD` e `Device CRD` con stato reported.
- Mappatura eventi runtime -> status Kubernetes (es. Running/Unreachable).
- Aggiornamenti heartbeat con cadenza periodica.

### 3.1.5 Emulazione Renode

- Caso single-device consolidato.
- Evoluzione verso manager centralizzato (un container Renode con N macchine virtuali) documentata e avviata.

---

## 3.2 Asset gia disponibili dal filone SPIN/OCRE parallelo

Dalle attivita della repo di tesi e delle analisi svolte:

- Packaging e deploy di applicazioni WASM cloud-oriented (SPIN, OCI image, deploy in Kubernetes).
- Build edge-oriented WASM (target WASI), conversione payload per integrazione firmware.
- Documentazione architetturale, guide operative e demo flow edge-to-cloud.

Questi asset possono essere riutilizzati in Retrospect come blueprint tecnico e materiale sperimentale.

---

## 3.3 Gap dichiarati e/o ancora da chiudere

1. Integrazione firmware standard (riduzione fork/custom branch).
2. Verifica end-to-end forte: `DeployAck` + esecuzione reale modulo + stato CRD sempre consistente nel tempo.
3. Osservabilita applicativa: metriche workload e logging runtime edge pubblicati in modo strutturato.
4. Multi-device e test di scalabilita (non solo caso singolo emulato).
5. Unificazione completa SPIN + OCRE nel control-plane Retrospect (policy, scheduling, artifact model unico).

---

## 4) Visione target: novelty da raggiungere

## 4.1 Problema scientifico/ingegneristico

Come orchestrare workload WebAssembly su infrastruttura eterogenea cloud-fog-edge con runtime diversi (SPIN e OCRE/WAMR) mantenendo:

- un solo piano di controllo;
- semantica di deploy uniforme;
- stato operativo affidabile e osservabile.

## 4.2 Novelty proposta

### "Single control-plane, dual runtime-plane for WebAssembly continuum"

Retrospect diventa capace di:

1. Ricevere intent unico di deploy (CRD).
2. Decidere target runtime (`spin`, `ocre`, `auto`).
3. Tradurre automaticamente il deploy nel formato/flow adatto al runtime.
4. Riallineare lo stato reale di esecuzione in un modello CRD unificato.

Questa novelty e rilevante per pubblicazione/tesi perche combina orchestration Kubernetes, edge constrained runtime e coerenza di stato cross-runtime.

---

## 5) Architettura target (dettagliata)

## 5.1 Nuovo modello logico

### 5.1.1 Control-plane unico (Retrospect)

- API Server
- Gateway
- Controller CRD
- Persistence/status model

### 5.1.2 Execution-plane duale

- SPIN plane (cloud/fog): componenti WASM eseguiti su runtime SPIN.
- OCRE plane (edge): moduli WASM eseguiti in WAMR su Zephyr/MCU.

## 5.2 Runtime Adapter Layer nel Gateway

Il Gateway introduce adapter espliciti:

- `SpinRuntimeAdapter`
  - prepara deploy verso pipeline SPIN
  - raccoglie ack/stato da lato cloud runtime
- `OcreRuntimeAdapter`
  - invia DeployApplication via TLS/CBOR
  - gestisce ack/status heartbeat edge
  - normalizza errori runtime/transport

## 5.3 CRD unificato (proposta minima)

### 5.3.1 Campi nuovi in Application spec

- `runtimeTarget`: `spin | ocre | auto`
- `artifact`:
  - `spinImage` (opzionale)
  - `wasmBytes` o riferimento artifact edge
- `placementPolicy`:
  - `latencySensitive`
  - `resourceProfile`
  - `fallbackMode`

### 5.3.2 Campi status unificati

- `phase` globale (Pending/Deploying/Running/Degraded/Failed)
- `runtimeStatus` per target runtime
- `deviceStatuses` (edge per-device)
- `cloudStatuses` (pod/runtime instances)
- `lastHeartbeat`, `lastError`, `lastTransitionTime`

---

## 6) Piano implementativo molto dettagliato

## 6.1 Workstream A - Model & API

### Obiettivo

Evolvere CRD/API senza rompere compatibilita.

### Attivita

1. Estendere schema Application CRD con campi runtime/placement.
2. Aggiornare validazione e defaulting.
3. Versionare API se necessario (`v1alpha1` -> `v1beta1` o compat layer).
4. Aggiornare DTO nel gateway e controller.

### Done criteria

- CRD applicabile in cluster pulito.
- Oggetti legacy ancora accettati.
- Test di serializzazione/deserializzazione superati.

---

## 6.2 Workstream B - Gateway Runtime Adapter Layer

### Obiettivo

Isolare logica runtime-specifica e centralizzare orchestrazione.

### Attivita

1. Introdurre trait/interfaccia comune:
   - `deploy()`
   - `stop()`
   - `query_status()`
   - `collect_metrics()`
2. Implementare adapter OCRE usando flussi gia stabilizzati.
3. Implementare adapter SPIN con integrazione cluster runtime.
4. Aggiungere orchestratore che seleziona adapter in base a `runtimeTarget`.
5. Gestione error taxonomy comune (transport/protocol/runtime/config).

### Done criteria

- Deploy test `runtimeTarget=ocre` e `runtimeTarget=spin` entrambi funzionanti.
- Stato CRD aggiornato con stessa semantica.

---

## 6.3 Workstream C - Artifact pipeline duale

### Obiettivo

Rendere ripetibile la creazione artifact per entrambi i runtime.

### Attivita

1. Definire naming/versioning unificato artifact.
2. Pipeline build SPIN (OCI).
3. Pipeline build OCRE (WASI/WAMR compatible payload).
4. Introduzione manifest "logical app -> physical artifacts".
5. Script CI/CD per pubblicazione artifact.

### Done criteria

- Da una sorgente app si generano artifact validi per entrambi i target.
- Deploy automatico senza passaggi manuali non documentati.

---

## 6.4 Workstream D - Observability e stato coerente

### Obiettivo

Dimostrare che lo stato Kubernetes rappresenta stato runtime reale.

### Attivita

1. Telemetria minima standard:
   - `app_started`
   - `app_running`
   - `app_failed`
   - `heartbeat`
2. Correlazione eventi deploy-id / app-id / device-id.
3. Aggiornamento CRD idempotente e robusto.
4. Dashboard/endpoint per verifica rapida coerenza.

### Done criteria

- Coerenza verificata in test nominali e failure scenarios.
- Nessun falso "Running" quando runtime edge non esegue davvero.

---

## 6.5 Workstream E - Multi-device e resilienza

### Obiettivo

Scalare oltre il caso single-device.

### Attivita

1. Test N device su Renode manager.
2. Code/backpressure gateway.
3. Retry policy e timeouts adattivi.
4. Recovery su reconnect device.

### Done criteria

- Scenari con N device stabili.
- Nessuna perdita silenziosa di deploy/status.

---

## 7) Piano sperimentale per validazione tesi

## 7.1 Domande sperimentali

1. Il control-plane unico gestisce correttamente dual runtime eterogenei?
2. Lo stato CRD resta coerente con stato runtime reale in condizioni nominali e degradate?
3. Il placement policy-aware migliora obiettivi (latenza/risorse/affidabilita)?

## 7.2 Esperimenti minimi obbligatori

### Esperimento E1 - Functional parity

- stessa applicazione logica su SPIN e OCRE;
- verifica risultato funzionale equivalente.

### Esperimento E2 - Status fidelity

- confronto stato osservato runtime vs stato CRD;
- misura mismatch rate.

### Esperimento E3 - Failure & recovery

- disconnessione device, restart gateway, timeout rete;
- misura tempo di convergenza e correttezza stato finale.

### Esperimento E4 - Multi-device

- incremento progressivo numero device emulati;
- misura throughput deploy/status e stabilita.

## 7.3 KPI consigliati

- Deployment success rate (%)
- Mean deploy latency (s)
- Ack-to-running latency (s)
- Status mismatch rate (%)
- Heartbeat loss rate (%)
- Recovery time after disconnect (s)
- CPU/RAM gateway sotto carico

---

## 8) Rischi principali e mitigazioni

## 8.1 Rischi tecnici

1. Divergenza semantica SPIN vs OCRE
   - Mitigazione: adapter layer + contract test comuni.
2. Stato inconsistente tra runtime e CRD
   - Mitigazione: eventi idempotenti + reconciliation loop.
3. Overhead gateway con multi-device
   - Mitigazione: backpressure, batching, profiling.
4. Firmware fork troppo custom
   - Mitigazione: piano di upstream/rebase su firmware standard.

## 8.2 Rischi progettuali

1. Scope creep (troppe feature accessorie)
   - Mitigazione: MVP chiaro e milestone a gate.
2. Debito documentale
   - Mitigazione: aggiornamento documentazione per ogni milestone chiusa.

---

## 9) Roadmap proposta (6-8 settimane)

## Settimana 1-2

- CRD esteso + compatibilita.
- Scheletro adapter layer.
- test e2e ocre consolidati.

## Settimana 3-4

- Adapter SPIN integrato.
- artifact duale automatizzato.
- primi test parity SPIN/OCRE.

## Settimana 5-6

- observability completa e status fidelity.
- failure scenarios e recovery tests.
- multi-device base.

## Settimana 7-8 (buffer e rifinitura)

- hardening, profiling, ottimizzazione.
- raccolta risultati sperimentali.
- stesura capitolo novelty + discussione limiti/futuro.

---

## 10) Deliverable finali attesi

1. Estensione CRD e API documentata.
2. Gateway con Runtime Adapter Layer SPIN + OCRE.
3. Pipeline artifact duale ripetibile.
4. Test suite e2e (nominale + failure + multi-device).
5. Dashboard/stato osservabile coerente.
6. Report sperimentale con KPI e confronto.

---

## 11) Criteri di accettazione ("definition of done" tesi/prototipo)

Il prototipo si considera completo quando:

1. Deploy `runtimeTarget=spin` e `runtimeTarget=ocre` funzionano da stesso control-plane.
2. Il runtime edge esegue realmente il modulo e invia stato periodico verificabile.
3. Lo stato Kubernetes riflette il runtime reale con mismatch trascurabile.
4. Almeno un test multi-device e un test failure-recovery sono superati.
5. Tutto e documentato in modo riproducibile (setup, runbook, risultati).

---

## 12) Cosa e gia "capitalizzabile" subito

Per accelerare il percorso, si possono riusare immediatamente:

- fix TLS/enrollment e framing robusto gia implementati;
- deploy path con ack e status periodico lato firmware;
- base cloud SPIN gia testata in ambiente Kubernetes;
- documentazione tecnica prodotta nelle ultime iterazioni.

In pratica, la parte piu difficile (far parlare davvero i blocchi critici) e gia stata affrontata; il prossimo salto e l'unificazione architetturale rigorosa e la validazione sperimentale strutturata.

---

## 13) Prossimi passi operativi immediati (azione entro 48h)

1. Congelare baseline attuale in branch dedicato integrazione.
2. Aprire issue/milestone per i 5 workstream (A-E) con owner e scadenze.
3. Definire schema CRD esteso e adapter interface in bozza.
4. Eseguire un primo e2e "dual target" con app minimale.
5. Aggiornare dashboard/logging per misurare i KPI base.

Con questi passi si passa dalla fase "fix e consolidamento" alla fase "novelty dimostrabile".

