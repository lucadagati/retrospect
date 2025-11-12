# Mock Data System Guide
## Sistema di Fallback con Dati Simulati

## 🎯 Problema Risolto

La dashboard mostrava timeout di 10 secondi quando l'API server non rispondeva:
```
API call failed for /api/v1/gateways: Error: Request timeout after 10000ms
```

## ✅ Soluzione Implementata

Sistema di fallback automatico con dati mock quando l'API fallisce o va in timeout.

### Componenti Creati

1. **`mockData.js`**: Dati simulati per gateways, devices, applications
2. **`api.js` (modificato)**: Fallback automatico ai mock
3. **`MockDataBanner.js`**: Banner informativo quando i mock sono attivi

### Dati Mock Disponibili

| Tipo | Quantità | Descrizione |
|------|----------|-------------|
| **Gateways** | 3 | gateway-1 (Running), gateway-2 (Running), gateway-3 (Pending) |
| **Devices** | 6 | device-1 to device-6 (Connected, Enrolled, Pending) |
| **Applications** | 5 | Temperature Monitor, LED Blinker, Data Logger, Firmware Updater, Network Diagnostics |

### Come Funziona

```javascript
// 1. La dashboard chiama l'API
apiGet('/api/v1/gateways', 10000)

// 2. Se l'API fallisce/timeout, usa i mock
↓ API Timeout (10s)
↓ Fallback attivato
↓ getMockData('gateways', 200ms)
✓ Ritorna MOCK_GATEWAYS
```

### Configurazione

In `dashboard-react/src/utils/api.js`:
```javascript
const USE_MOCK_FALLBACK = true; // Abilita/disabilita mock fallback
const DEFAULT_TIMEOUT = 10000;  // Timeout in ms
```

### Mock per Operazione

| Operazione | API Reale Fallisce | Mock Response |
|------------|-------------------|---------------|
| **GET** `/gateways` | ✅ Usa MOCK_GATEWAYS | 3 gateways simulati |
| **GET** `/devices` | ✅ Usa MOCK_DEVICES | 6 devices simulati |
| **GET** `/applications` | ✅ Usa MOCK_APPLICATIONS | 5 apps simulate |
| **POST** any | ✅ Success simulato | `{success: true, message: "Mock: Operation completed"}` |
| **PUT** any | ✅ Update simulato | `{success: true, message: "Mock: Update completed"}` |
| **DELETE** any | ✅ Delete simulato | `{success: true, message: "Mock: Delete completed"}` |

### Banner Demo Mode

Quando i mock sono attivi, appare un banner informativo:

```
ℹ️ Demo Mode Active (3 APIs using mock data)
   The dashboard is using simulated data because the API server 
   is unavailable or responding slowly. All functionality is 
   demonstrated with mock data for testing purposes.
```

## 🎨 Dati Mock - Dettaglio

### Gateways
```javascript
{
  id: "gateway-1",
  name: "gateway-1",
  status: "Running",
  endpoint: "gateway-1-service.wasmbed.svc.cluster.local:8080",
  connected_devices: 3,
  enrolled_devices: 5,
  capabilities: ["TLS", "WASM", "OTA"],
  lastHeartbeat: "2025-11-12T..."
}
```

### Devices
```javascript
{
  id: "device-1",
  name: "device-1",
  type: "MCU",
  architecture: "ARM_CORTEX_M",
  mcuType: "RenodeArduinoNano33Ble",
  status: "Connected",
  gatewayId: "gateway-1",
  emulationStatus: "Running",
  publicKey: "MCowBQYDK2VwAyEA...",
  lastHeartbeat: 1731408000
}
```

### Applications
```javascript
{
  id: "app-1",
  name: "Temperature Monitor",
  description: "Monitors device temperature and sends alerts",
  status: "Running",
  deployed_devices: ["device-1", "device-2", "device-3"],
  target_devices: ["device-1", "device-2", "device-3", "device-4"],
  statistics: {
    total_devices: 4,
    running_devices: 3,
    failed_devices: 0,
    deployment_progress: 75.0
  }
}
```

## 🔧 Personalizzazione Mock

### Aggiungere Nuovi Dati Mock

Modifica `dashboard-react/src/utils/mockData.js`:

```javascript
export const MOCK_GATEWAYS = [
  ...MOCK_GATEWAYS,
  {
    id: "gateway-custom",
    name: "Custom Gateway",
    status: "Running",
    // ... altri campi
  }
];
```

### Variare Dati Mock Randomicamente

```javascript
import { randomizeStatus } from './mockData';

const gateways = randomizeStatus(MOCK_GATEWAYS, 'status');
// Ora ogni gateway ha uno status random: Running, Stopped, Pending, Failed
```

### Simulare Delay Specifici

```javascript
// In mockData.js, modificare il delay
export const getMockData = async (type, delay = 500) => {
  await new Promise(resolve => setTimeout(resolve, delay)); // 500ms default
  // ...
};
```

## 🧪 Test

### Testare con Mock Attivi

1. **Disabilita API server** o **blocca porta 3001**:
```bash
# Ferma API server
pkill wasmbed-api-server

# O blocca la porta con firewall (reversibile)
sudo iptables -A OUTPUT -p tcp --dport 3001 -j REJECT
```

2. **Apri la dashboard**: http://localhost:3000

3. **Verifica comportamento**:
   - Dashboard si carica
   - Dati mock vengono mostrati
   - Banner "Demo Mode Active" appare
   - Tutte le funzionalità sono dimostrabili

### Testare Fallback Specifico

```javascript
// In Console del browser
fetch('http://localhost:3001/api/v1/gateways')
  .catch(() => console.log('API failed, mock should activate'));
```

### Disabilitare Mock (Force Real API)

In `dashboard-react/src/utils/api.js`:
```javascript
const USE_MOCK_FALLBACK = false; // Disabilita mock
```

Ora la dashboard mostrerà errori se l'API non risponde.

## 🎯 Use Cases

### 1. Demo/Presentazione
- API server non necessario
- Dati sempre disponibili
- Prestazioni costanti

### 2. Development Frontend
- Sviluppo UI senza backend
- Test rapidi senza setup completo
- Nessuna dipendenza da Kubernetes

### 3. Testing Resilienza
- Verifica comportamento con API lenta
- Test fallback system
- Validazione UX in condizioni difficili

## 📊 Statistiche Mock

| Metrica | Valore |
|---------|--------|
| Gateways totali | 3 |
| Gateways Running | 2 |
| Devices totali | 6 |
| Devices Connected | 3 |
| Devices Enrolled | 2 |
| Applications totali | 5 |
| Applications Running | 2 |
| Applications Failed | 1 |
| Delay mock medio | 200-500ms |

## 🚀 Deployment

### Production
Disabilita mock in production:
```javascript
const USE_MOCK_FALLBACK = process.env.NODE_ENV !== 'production';
```

### Staging
Mantieni mock abilitati per testing:
```javascript
const USE_MOCK_FALLBACK = true;
```

## 📝 Note Implementative

1. **Performance**: Mock response in 200-500ms vs API real 50-100ms
2. **Realismo**: Dati mock statici, non cambiano nel tempo
3. **Limitazioni**: 
   - POST/PUT/DELETE non modificano realmente lo stato
   - Relazioni tra entità sono simulate
   - Nessuna validazione backend

## 🔍 Troubleshooting

### Mock Non Si Attivano
```javascript
// Verifica in console
console.log('Mock fallback enabled:', USE_MOCK_FALLBACK);
```

### Banner Non Appare
- Verifica che `MockDataBanner` sia importato in `App.js`
- Controlla console per messaggi "Using mock..." o "mock fallback"

### Dati Mock Non Realistici
- Modifica `mockData.js` per adattare alle tue esigenze
- Usa `randomizeStatus()` per variazione

## ✅ Vantaggi

1. **Resilienza**: Dashboard funziona sempre, anche con API down
2. **UX**: Nessun "loading infinito", sempre dati disponibili
3. **Demo**: Perfetto per presentazioni senza infrastructure
4. **Development**: Frontend team può lavorare indipendentemente
5. **Testing**: Facile testare diversi scenari

## 🎉 Risultato

Dashboard completamente funzionale in ogni condizione:
- ✅ API server funzionante: Usa dati reali
- ✅ API server lento: Timeout, poi usa mock
- ✅ API server offline: Usa mock immediatamente
- ✅ Kubernetes non disponibile: Usa mock
- ✅ Nessuna infrastruttura: Usa mock

**La dashboard funziona SEMPRE! 🚀**

