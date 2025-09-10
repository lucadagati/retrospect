# 🚁 Wasmbed Platform Dashboard

Una dashboard completa e moderna per la gestione e il monitoraggio del sistema Wasmbed Platform - un middleware per il deployment sicuro di applicazioni WebAssembly su sistemi robotici industriali con orchestrazione Kubernetes.

![Dashboard Preview](https://img.shields.io/badge/Status-In%20Development-yellow)
![React](https://img.shields.io/badge/React-18.2.0-blue)
![Material-UI](https://img.shields.io/badge/Material--UI-5.14.20-purple)
![Three.js](https://img.shields.io/badge/Three.js-0.158.0-green)
![License](https://img.shields.io/badge/License-AGPL--3.0-red)

## 🎯 **Panoramica**

Questa dashboard fornisce un'interfaccia web moderna per gestire:
- **Dispositivi edge** (droni, gateway, sensori)
- **Applicazioni WASM** deployate sui dispositivi
- **Cluster Kubernetes** per l'orchestrazione
- **Controllo drone 3D** in tempo reale
- **Monitoring sistema** completo

## ✨ **Funzionalità Implementate**

### ✅ **Dashboard Overview**
- [x] **Cards statistiche** con metriche sistema
- [x] **Grafici real-time** per CPU, Memory, Network
- [x] **Sistema di alert** con severità e timestamp
- [x] **Activity feed** per eventi recenti
- [x] **Pie chart** per distribuzione risorse

### ✅ **Controllo Drone 3D**
- [x] **Visualizzazione 3D** del drone con Three.js
- [x] **Modello dettagliato** (corpo, gimbal, LED, motori, landing gear)
- [x] **Ambiente realistico** (terreno, griglia, illuminazione, sky dome)
- [x] **Controlli di volo** (arm/disarm, takeoff, land, hover, emergency)
- [x] **Telemetria completa** (posizione, attitude, batteria)
- [x] **Controllo altitudine** con slider e markers

### ✅ **Device Management**
- [x] **Tabella dispositivi** con status real-time
- [x] **Device statistics** (totale, connessi, offline, enrolling)
- [x] **Add device dialog** per enrollment
- [x] **Actions** (edit, delete, refresh)

### ✅ **Application Management**
- [x] **Tabella applicazioni WASM** con metriche
- [x] **Memory/CPU usage** indicators
- [x] **Application lifecycle** (deploy, start, stop, delete)
- [x] **Version management** e restart count

### ✅ **UI/UX Avanzata**
- [x] **Layout responsive** con sidebar collassabile
- [x] **Tema Material-UI** personalizzato
- [x] **Animazioni e transizioni** fluide
- [x] **Indicatori di connessione** WebSocket
- [x] **Dark/Light theme** support

## 🚧 **Roadmap - Cosa Implementare**

### 🔄 **Backend Integration (Priorità Alta)**
- [ ] **WebSocket real-time** - Sostituire simulazione con connessioni reali
- [ ] **API REST integration** - Collegare alle API del gateway Wasmbed
- [ ] **Authentication/Authorization** - Sistema login con JWT
- [ ] **Error handling** - Gestione errori API e retry logic

### 📊 **Monitoring Avanzato (Priorità Alta)**
- [ ] **Prometheus integration** - Metriche real-time da Prometheus
- [ ] **Grafana embedding** - Embed dashboard Grafana esistenti
- [ ] **Custom alerts** - Sistema alerting configurabile
- [ ] **Log aggregation** - Visualizzazione log centralizzati
- [ ] **Performance analytics** - Analisi trend e predizioni

### 🚁 **Controllo Drone Avanzato (Priorità Media)**
- [ ] **Mission planning** - Pianificazione missioni con waypoints
- [ ] **Flight path visualization** - Visualizzazione traiettorie 3D
- [ ] **Multiple drone support** - Gestione flotta droni
- [ ] **Camera feed integration** - Stream video in tempo reale
- [ ] **Geofencing** - Zone di volo sicure
- [ ] **Emergency protocols** - Procedure automatiche di emergenza

### 🔧 **Device Management Avanzato (Priorità Media)**
- [ ] **Device discovery** - Auto-discovery dispositivi in rete
- [ ] **Bulk operations** - Operazioni su più dispositivi
- [ ] **Device grouping** - Organizzazione per gruppi/tag
- [ ] **Configuration management** - Deploy configurazioni
- [ ] **Firmware updates** - OTA firmware updates
- [ ] **Device health checks** - Diagnostica automatica

### 📦 **Application Lifecycle (Priorità Media)**
- [ ] **WASM app store** - Repository applicazioni
- [ ] **Visual deployment** - Drag & drop deployment
- [ ] **Version rollback** - Rollback automatico
- [ ] **A/B testing** - Deploy graduale
- [ ] **Resource quotas** - Limiti risorse per app
- [ ] **Dependency management** - Gestione dipendenze

### 🔐 **Security & Compliance (Priorità Media)**
- [ ] **TLS certificate management** - Gestione certificati
- [ ] **Audit logging** - Log delle operazioni
- [ ] **Role-based access** - Controllo accessi granulare
- [ ] **Compliance dashboard** - Monitoraggio conformità
- [ ] **Security scanning** - Scansione vulnerabilità
- [ ] **Backup/Restore** - Backup configurazioni

### 🎨 **UI/UX Enhancements (Priorità Bassa)**
- [ ] **Custom dashboards** - Dashboard personalizzabili
- [ ] **Widget system** - Sistema widget modulare
- [ ] **Mobile app** - App mobile companion
- [ ] **Accessibility** - Supporto WCAG 2.1
- [ ] **Internationalization** - Supporto multi-lingua
- [ ] **Themes** - Temi personalizzabili

### 🔌 **Integrations (Priorità Bassa)**
- [ ] **Slack/Teams notifications** - Notifiche chat
- [ ] **Email alerts** - Alert via email
- [ ] **External APIs** - Integrazione API esterne
- [ ] **Export/Import** - Export dati e configurazioni
- [ ] **Webhook support** - Webhook per eventi
- [ ] **CI/CD integration** - Integrazione pipeline

## 🏗️ **Architettura Tecnica**

```
┌─────────────────────────────────────────────────────────────────┐
│                 WASMBED DASHBOARD v2.0                         │
├─────────────────────────────────────────────────────────────────┤
│  React 18 + Material-UI + Three.js + Recharts + WebSocket     │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌───────────┐  │
│  │  Overview   │ │ Drone 3D    │ │  Device     │ │   App     │  │
│  │ Dashboard   │ │  Control    │ │ Management  │ │Management │  │
│  └─────────────┘ └─────────────┘ └─────────────┘ └───────────┘  │
├─────────────────────────────────────────────────────────────────┤
│              WebSocket Context + State Management              │
├─────────────────────────────────────────────────────────────────┤
│                 HTTP API + WebSocket Gateway                   │
├─────────────────────────────────────────────────────────────────┤
│                    Wasmbed Gateway (Rust)                      │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌───────────┐  │
│  │ Kubernetes  │ │   Device    │ │    PX4      │ │  microROS │  │
│  │    API      │ │ Management  │ │Integration  │ │   Bridge  │  │
│  └─────────────┘ └─────────────┘ └─────────────┘ └───────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

## 🚀 **Quick Start**

### Prerequisiti
- Node.js 18+ (compatibile con versioni precedenti)
- npm 9+
- Browser moderno con supporto WebGL

### Installazione
```bash
git clone https://github.com/yourusername/wasmbed-dashboard.git
cd wasmbed-dashboard
npm install
```

### Sviluppo
```bash
npm run dev
```
Dashboard disponibile su: http://localhost:3000

### Build Produzione
```bash
npm run build
npm run preview
```

## 🔧 **Configurazione**

### Environment Variables
```bash
# .env.local
VITE_API_BASE_URL=http://localhost:8080
VITE_WEBSOCKET_URL=ws://localhost:8080/ws
VITE_ENABLE_MOCK_DATA=true
```

### Backend Integration
Per collegare al backend Wasmbed reale:
1. Configurare `VITE_API_BASE_URL` con l'URL del gateway
2. Impostare `VITE_ENABLE_MOCK_DATA=false`
3. Assicurarsi che le API siano accessibili (CORS)

## 📁 **Struttura Progetto**

```
wasmbed-dashboard/
├── src/
│   ├── components/           # Componenti React
│   │   ├── DashboardLayout.jsx
│   │   ├── OverviewDashboard.jsx
│   │   ├── DroneControlDashboard.jsx
│   │   ├── DevicesDashboard.jsx
│   │   ├── ApplicationsDashboard.jsx
│   │   ├── MonitoringDashboard.jsx
│   │   └── SettingsDashboard.jsx
│   ├── App.jsx              # App principale con Context
│   ├── main.jsx             # Entry point
│   └── index.css            # Stili globali
├── public/                  # Asset statici
├── package.json             # Dipendenze
├── vite.config.js          # Configurazione Vite
└── README.md               # Documentazione
```

## 🧪 **Testing**

```bash
# Unit tests (da implementare)
npm run test

# E2E tests (da implementare)
npm run test:e2e

# Linting
npm run lint
```

## 📊 **Tecnologie Utilizzate**

| Tecnologia | Versione | Scopo |
|------------|----------|-------|
| React | 18.2.0 | Framework UI |
| Material-UI | 5.14.20 | Componenti UI |
| Three.js | 0.158.0 | Grafica 3D |
| Recharts | 2.8.0 | Grafici |
| Vite | 4.5.1 | Build tool |
| Axios | 1.6.2 | HTTP client |

## 🤝 **Contribuire**

1. Fork del repository
2. Crea branch feature (`git checkout -b feature/amazing-feature`)
3. Commit modifiche (`git commit -m 'Add amazing feature'`)
4. Push branch (`git push origin feature/amazing-feature`)
5. Apri Pull Request

### Linee Guida
- Seguire convenzioni React/Material-UI
- Aggiungere test per nuove funzionalità
- Documentare API e componenti
- Mantenere compatibilità browser

## 📝 **Changelog**

### v2.0.0 (Gennaio 2025)
- ✨ Dashboard completa con 6 sezioni
- 🚁 Controllo drone 3D avanzato
- 📊 Grafici real-time e monitoring
- 🎨 UI/UX moderna con Material-UI
- 🔗 WebSocket integration (simulata)

### v1.0.0 (Gennaio 2025)
- 🎉 Prima versione con funzionalità base
- 📊 Dashboard overview
- 🚁 Controllo drone 3D base
- 🔧 Layout responsive

## 📄 **Licenza**

Questo progetto è licenziato sotto AGPL-3.0 License - vedi [LICENSE](LICENSE) per dettagli.

## 🙏 **Riconoscimenti**

- **Wasmbed Platform** - Sistema backend
- **Material-UI Team** - Componenti UI eccellenti
- **Three.js Community** - Grafica 3D potente
- **React Team** - Framework solido

---

**Status**: 🚧 **In Sviluppo Attivo**  
**Versione**: 2.0.0  
**Compatibilità**: Node.js 18+  
**Maintainer**: [@lucadag](https://github.com/lucadag)  
**Ultimo aggiornamento**: Gennaio 2025

---

### 🔗 **Link Utili**
- [Wasmbed Platform](https://github.com/yourusername/wasmbed-platform)
- [Documentazione API](./docs/api.md)
- [Guida Sviluppatori](./docs/development.md)
- [Roadmap Dettagliata](./docs/roadmap.md)