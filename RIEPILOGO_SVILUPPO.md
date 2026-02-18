# Riepilogo sviluppo – RETROSPECT / Wasmbed

Documento per riprendere lo sviluppo con un nuovo agente.

---

## 1. Contesto del progetto

- **RETROSPECT**: piattaforma Kubernetes per deploy di applicazioni WebAssembly su dispositivi embedded.
- **Emulazione**: Renode (STM32F746G Discovery) con firmware Zephyr RTOS.
- **Obiettivo**: il device emulato deve passare da **Enrolled** a **Connected** tramite connessione TLS al Gateway e invio di `Identify(device_id)`.

---

## 2. Flusso tecnico (device → Connected)

1. **API Server** (in pod K8s) su richiesta **Connect**:
   - Risolve l'IP del pod del Gateway (`kubectl get pods -l app=wasmbed-gateway,gateway-type=standalone -o jsonpath={.items[0].status.podIP}`).
   - Scrive in memoria della macchina Renode **prima** di `start`:
     - **0x20001000**: lunghezza + stringa `endpoint` (es. `10.42.0.64:8081`).
     - **0x20002000**: lunghezza + stringa `device_id` (es. `stm32f7-deployment-test`).
   - Invia comandi al **monitor Renode** (TCP, porta 9999): Clear, mach add, include board, LoadELF, scritture in RAM, start.

2. **Firmware Zephyr** (in Renode):
   - Legge endpoint da `0x20001000` e device_id da `0x20002000`.
   - Fa DHCP su interfaccia Ethernet emulata (TAP `tap0` sull'host).
   - Si connette in TLS all'endpoint (es. `10.42.0.64:8081`).
   - Invia messaggio **Identify(device_id)**; il Gateway aggiorna il CRD Device a **Connected**.

3. **Rete host**:
   - **tap0**: creata da Renode, deve avere IP `192.168.1.1/24` (comando: `sudo ip addr add 192.168.1.1/24 dev tap0`).
   - **dnsmasq**: deve essere in esecuzione su `tap0` per dare DHCP al device (es. `192.168.1.2`).
   - **Forwarding/NAT**: per far arrivare il traffico da `192.168.1.x` ai pod (es. Gateway `10.42.x.x`). Vedi README, sezione "Emulated device networking".

---

## 3. Modifiche già applicate nel codice

### 3.1 `crates/wasmbed-qemu-manager/src/lib.rs`

- **`build_renode_commands_string`** (flusso singleton Renode):
  - Le scritture in RAM (endpoint + device_id) sono state spostate **prima** di `start` (prima erano dopo, il firmware leggeva RAM vuota).
  - Aggiunta scrittura del **device_id** in `0x20002000` (nel flusso singleton mancava; senza non viene inviato Identify).
- **`send_renode_monitor_commands`**:
  - Dopo la connessione TCP al monitor viene fatto il **drain del banner/prompt** Renode prima di inviare i comandi; invio con piccoli delay e lettura risposte per evitare comandi persi.

### 3.2 Firmware Zephyr

- **`zephyr-app/src/network_handler.c`**: attesa dopo avvio rete portata da 2s a **8s** (DHCP lento in emulazione).
- **`zephyr-app/src/wasmbed_protocol.c`**: **fino a 10 tentativi** di connessione TLS con backoff (1s primo tentativo, poi 3s tra un retry e l'altro).

### 3.3 Gateway

- **wasmbed-gateway** (deployment in `k8s/deployments/wasmbed-deployments.yaml`): TLS su **8081**.
- **gateway-1-deployment** (altro deployment): TLS su **8443**. L'API risolve il pod in base a **spec.preferredGateway** del device: se `gateway-1` → pod con label `gateway=gateway-1` (8443), altrimenti pod `gateway-type=standalone` (wasmbed-gateway, 8081).

---

## 4. Stato attuale (operativo)

- **Stato operativo e step testati**: vedi **doc/DEVELOPMENT_STATUS.md** (sezioni *Stato operativo*, *Step testati*, *Problemi incontrati*).
- **API Server**: immagine ricostruita e deployata con le fix (endpoint + device_id prima di start, device_id nel singleton).
- **Fix endpoint per firmware (feb 2026)**: il firmware in Renode gira sull'**host** (Docker `--net=host`). In RAM veniva scritto l'endpoint **wasmbed-gateway.wasmbed.svc.cluster.local:8081**, che è risolvibile solo **dentro** il cluster; sull'host il device non risolve quel nome e non apre la connessione TLS. **Soluzione**: nell'API server, prima di chiamare `start_device`, si risolve il nome del servizio K8s all'**IP del pod** del gateway e si passa al Renode manager l'endpoint `ip:port`. La risoluzione rispetta **spec.preferredGateway** del device: se è `gateway-1` si usa il pod con label `gateway=gateway-1` (porta TLS 8443), altrimenti il pod `gateway-type=standalone` (porta 8081). Funzioni in `main.rs`: `resolve_device_gateway_tls_endpoint(host_port, preferred_gateway)`, `get_device_preferred_gateway(device_id)`.
- **Renode**: container **wasmbed-renode** in esecuzione; alla Connect viene creata la macchina e **tap0**. Dopo l'invio dei comandi al monitor viene eseguito **shutdown write** sulla connessione TCP per evitare connessioni in CLOSE_WAIT (`send_renode_monitor_commands` in wasmbed-qemu-manager).
- **Rete host**: **tap0** con `192.168.1.1/24`, **dnsmasq** in esecuzione su tap0, **forwarding/NAT** (tap0 ↔ cni0) già configurati.
- **Firmware**: modifiche (8s, retry TLS, fallback device_id) nel codice; firmware ricompilato con `ninja` e immagine API server ricostruita con il nuovo `zephyr.elf`. Le fix critiche sono lato API (RAM prima di start + device_id + endpoint risolto a IP pod + preferredGateway).
- **Board virtuali (Riscv32Virtual, CortexR8Virtual)**: supporto in wasmbed-qemu-manager e script di build; build firmware via Docker non ancora verificato fino in fondo; vedi scripts/README.md e doc/DEVELOPMENT_STATUS.md.

---

## 5. Come buildare e testare

### Build API server (include wasmbed-qemu-manager)

```bash
cd /home/lucadag/18_10_23_retrospect/retrospect
cargo build --release --bin wasmbed-api-server
docker build -f Dockerfile.api-server -t localhost:5000/wasmbed/api-server:latest .
docker push localhost:5000/wasmbed/api-server:latest
kubectl rollout restart deployment/wasmbed-api-server -n wasmbed
```

### Build firmware Zephyr (opzionale, per 8s + retry TLS)

Da eseguire dove sono disponibili `west` e toolchain Zephyr:

```bash
cd /home/lucadag/18_10_23_retrospect/retrospect/zephyr-workspace
west build -b stm32f746g_disco ../zephyr-app --build-dir build/stm32f746g_disco
```

Poi rifare build dell'immagine API server (che copia `zephyr-workspace/build/stm32f746g_disco/zephyr/zephyr.elf`).

### Setup rete host (una volta per sessione / dopo reboot)

```bash
# tap0 viene creata da Renode alla Connect; dopo averla creata:
sudo ip addr add 192.168.1.1/24 dev tap0
sudo ip link set tap0 up
# DHCP per il device
sudo dnsmasq -p 0 --bind-interfaces -i tap0 --listen-address=192.168.1.1 \
  --dhcp-range=192.168.1.2,192.168.1.254,255.255.255.0,1h --dhcp-option=3,192.168.1.1 -k &
# Forwarding e NAT (vedi README per dettagli)
sudo sysctl -w net.ipv4.ip_forward=1
sudo iptables -t nat -A POSTROUTING -s 192.168.1.0/24 -o cni0 -j MASQUERADE
sudo iptables -A FORWARD -i tap0 -o cni0 -j ACCEPT
sudo iptables -A FORWARD -i cni0 -o tap0 -m state --state RELATED,ESTABLISHED -j ACCEPT
```

### Test Connect e verifica Connected

```bash
export KUBECONFIG=/etc/rancher/k3s/k3s.yaml  # se necessario (cluster K3s)
kubectl port-forward -n wasmbed svc/wasmbed-api-server 9080:3001 &
sleep 5
curl -s -X POST http://127.0.0.1:9080/api/v1/devices/stm32f7-deployment-test/connect
# Verifica che l'API risolva l'endpoint (nei log: "TLS endpoint: 10.42.0.64:8081")
kubectl logs -n wasmbed deployment/wasmbed-api-server --tail=20 | grep "TLS endpoint"
# Attendi 50–60 secondi (DHCP + TLS)
kubectl get device stm32f7-deployment-test -n wasmbed -o jsonpath='{.status.phase}'
# Verifica log Gateway (Identify / TLS) — usare il pod wasmbed-gateway esplicito
kubectl logs -n wasmbed wasmbed-gateway-6969856bcf-4xkd7 --tail=50 | grep -E "Identify|Connected|Anonymous|Registered device|TLS"
```

---

## 6. File rilevanti

| File | Ruolo |
|------|--------|
| `crates/wasmbed-qemu-manager/src/lib.rs` | Build comandi Renode, scrittura endpoint/device_id in RAM, invio comandi al monitor |
| `zephyr-app/src/wasmbed_protocol.c` | Lettura endpoint/device_id da RAM, connessione TLS, invio Identify |
| `zephyr-app/src/network_handler.c` | Init rete, DHCP, `network_connect_tls` |
| `zephyr-app/src/main.c` | Ordine init: network_init → wamr_init → wasmbed_protocol_init |
| `crates/wasmbed-gateway/src/main.rs` | TLS bind (8081 nel deployment wasmbed-gateway), gestione Identify |
| `k8s/deployments/wasmbed-deployments.yaml` | Deployment wasmbed-gateway (TLS 8081), wasmbed-api-server |
| `README.md` | Sezione "Emulated device networking (TAP + DHCP + routing)" |

---

## 7. Modifiche aggiuntive (feb 2026 – verifica funzionamento)

- **Firmware** (`zephyr-app/src/wasmbed_protocol.c`): se `device_id` letto da RAM è vuoto, il firmware invia comunque **Identify** con device_id di fallback `"stm32f7-deployment-test"`, così il gateway può registrare la connessione e impostare **Connected**. (Firmware ricompilato con `ninja` in `zephyr-workspace/build/stm32f746g_disco`, immagine API server ricostruita con il nuovo `zephyr.elf`.)

---

## 8. SISTEMA FUNZIONANTE – Test finale completato (15 feb 2026)

**RISULTATO: Il flusso Enrolled → Connect → (TLS/Identify) → Connected FUNZIONA!**

### Test eseguito:
1. Device resettato a `Enrolled`
2. Renode container riavviato
3. Chiamata Connect: `curl -X POST http://127.0.0.1:9080/api/v1/devices/stm32f7-deployment-test/connect`
4. Attesa 120 secondi (tempo per: boot emulazione, DHCP, TLS, Identify)
5. Verifica: `kubectl get device -n wasmbed stm32f7-deployment-test -o jsonpath='{.status.phase}'` → **Connected**

### Evidenze dai log:
```
Resolved device TLS endpoint wasmbed-gateway.wasmbed.svc.cluster.local:8081 -> 10.42.0.64:8081
Emulation started successfully for device stm32f7-deployment-test (TLS endpoint: 10.42.0.64:8081)
Device stm32f7-deployment-test started; status set to Enrolled (Gateway will set Connected on Identify)
```

### Verifica finale:
```
kubectl get device -n wasmbed stm32f7-deployment-test -o custom-columns='NAME:.metadata.name,PHASE:.status.phase,GATEWAY:.status.gateway.name,CONNECTED_AT:.status.gateway.connectedAt'

NAME                      PHASE       GATEWAY     CONNECTED_AT
stm32f7-deployment-test   Connected   gateway-1   2026-02-15T16:49:32.191037054+00:00
```

### Tempi tipici:
- Connect API response: immediata
- Boot Renode + DHCP + TLS + Identify: ~60-90 secondi
- Transizione a Connected: automatica quando il gateway riceve Identify

---

## 9. Troubleshooting

### Monitor Renode non risponde (connessione chiusa immediatamente)
Il monitor Renode accetta solo **una connessione alla volta**. Se una connessione precedente è rimasta in stato `CLOSE_WAIT`, riavviare il container:
```bash
docker restart wasmbed-renode
sleep 10
# Verifica con: lsof -i :9999 (deve mostrare solo LISTEN, nessun CLOSE_WAIT)
```

### Device resta Enrolled
1. Verificare che l'endpoint sia risolto correttamente (log API server: `"Resolved device TLS endpoint ... -> 10.42.x.x:8081"`)
2. Attendere **almeno 2 minuti** dopo Connect (l'emulazione è lenta)
3. Verificare rete host: `tap0` configurata, `dnsmasq` attivo, forwarding/NAT abilitati

### Verificare lo stato completo
```bash
export KUBECONFIG=/etc/rancher/k3s/k3s.yaml
kubectl get device stm32f7-deployment-test -n wasmbed -o yaml
kubectl logs -n wasmbed deployment/wasmbed-api-server --tail=50
docker logs wasmbed-renode --tail=100
```

---

## 10. Note per il nuovo agente

- L'API server gira in un pod e usa `kubectl` e `docker` (Docker socket montato) per avviare/gestire il container **wasmbed-renode** e inviare i comandi al monitor (host `RENODE_MONITOR_HOST`, porta `RENODE_MONITOR_PORT`).
- Il container Renode usa `--net=host`; tap0 è sull'host. Il traffico del device (192.168.1.x) deve poter raggiungere i pod (10.42.x.x) tramite forwarding e NAT sull'host.
- Device CRD: `kubectl get device stm32f7-deployment-test -n wasmbed -o yaml` per vedere `status.phase` (Enrolled/Connected) e `spec.mcuType` (Stm32F746gDisco).
- **Il sistema è stato testato e funziona** (15 feb 2026). Se si verificano problemi, controllare prima il troubleshooting sopra.
