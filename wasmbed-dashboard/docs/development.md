# 🛠️ Development Guide - Wasmbed Dashboard

Guida completa per sviluppatori che vogliono contribuire al progetto Wasmbed Dashboard.

## 🎯 **Quick Start**

### Prerequisiti
- **Node.js** 18+ (compatibile con versioni precedenti)
- **npm** 9+
- **Git** per version control
- **Browser moderno** con supporto WebGL per visualizzazione 3D

### Setup Ambiente
```bash
# Clone repository
git clone https://github.com/yourusername/wasmbed-dashboard.git
cd wasmbed-dashboard

# Install dependencies
npm install

# Start development server
npm run dev
```

Dashboard disponibile su: http://localhost:3000

## 📁 **Struttura Progetto**

```
wasmbed-dashboard/
├── src/
│   ├── components/           # Componenti React
│   │   ├── DashboardLayout.jsx      # Layout principale
│   │   ├── OverviewDashboard.jsx    # Dashboard overview
│   │   ├── DroneControlDashboard.jsx # Controllo drone 3D
│   │   ├── DevicesDashboard.jsx     # Gestione dispositivi
│   │   ├── ApplicationsDashboard.jsx # Gestione applicazioni
│   │   ├── MonitoringDashboard.jsx  # Monitoring avanzato
│   │   └── SettingsDashboard.jsx    # Impostazioni
│   ├── hooks/                # Custom React hooks
│   ├── utils/                # Utility functions
│   ├── services/             # API services
│   ├── contexts/             # React contexts
│   ├── App.jsx               # App principale
│   ├── main.jsx              # Entry point
│   └── index.css             # Stili globali
├── public/                   # Asset statici
├── docs/                     # Documentazione
├── tests/                    # Test files
├── package.json              # Dependencies
├── vite.config.js           # Vite configuration
└── README.md                # Main documentation
```

## 🧩 **Architettura Componenti**

### Component Hierarchy
```
App (Context Provider)
├── DashboardLayout (Sidebar + AppBar)
│   ├── OverviewDashboard
│   │   ├── StatCard
│   │   ├── MetricsChart
│   │   └── AlertList
│   ├── DroneControlDashboard
│   │   ├── ThreeJSViewer
│   │   ├── FlightControls
│   │   └── TelemetryPanel
│   ├── DevicesDashboard
│   │   ├── DeviceTable
│   │   └── AddDeviceDialog
│   └── ApplicationsDashboard
│       ├── AppTable
│       └── DeploymentDialog
```

### State Management
- **React Context** per stato globale (WebSocket data)
- **useState/useEffect** per stato locale componenti
- **Custom hooks** per logica riutilizzabile

## 🎨 **Design System**

### Theme Configuration
```javascript
// src/theme.js
import { createTheme } from '@mui/material/styles';

const theme = createTheme({
  palette: {
    primary: { main: '#1976d2' },
    secondary: { main: '#dc004e' },
    background: { default: '#f5f5f5' }
  },
  typography: {
    fontFamily: 'Roboto, sans-serif'
  },
  components: {
    MuiCard: {
      styleOverrides: {
        root: {
          borderRadius: '12px',
          boxShadow: '0 2px 8px rgba(0,0,0,0.1)'
        }
      }
    }
  }
});
```

### Color Palette
- **Primary**: #1976d2 (Blue)
- **Secondary**: #dc004e (Pink)
- **Success**: #4caf50 (Green)
- **Warning**: #ff9800 (Orange)
- **Error**: #f44336 (Red)

### Typography
- **Font Family**: Roboto
- **Headings**: 600 weight
- **Body**: 400 weight
- **Captions**: 300 weight

## 🔌 **API Integration**

### Service Layer
```javascript
// src/services/api.js
import axios from 'axios';

const api = axios.create({
  baseURL: import.meta.env.VITE_API_BASE_URL || 'http://localhost:8080',
  timeout: 5000,
  headers: {
    'Content-Type': 'application/json'
  }
});

// Request interceptor for auth
api.interceptors.request.use((config) => {
  const token = localStorage.getItem('auth_token');
  if (token) {
    config.headers.Authorization = `Bearer ${token}`;
  }
  return config;
});

// Response interceptor for error handling
api.interceptors.response.use(
  (response) => response,
  (error) => {
    if (error.response?.status === 401) {
      // Handle unauthorized
      localStorage.removeItem('auth_token');
      window.location.href = '/login';
    }
    return Promise.reject(error);
  }
);

export default api;
```

### WebSocket Integration
```javascript
// src/services/websocket.js
class WebSocketService {
  constructor(url) {
    this.url = url;
    this.ws = null;
    this.listeners = new Map();
    this.reconnectAttempts = 0;
    this.maxReconnectAttempts = 5;
  }

  connect() {
    this.ws = new WebSocket(this.url);
    
    this.ws.onopen = () => {
      console.log('WebSocket connected');
      this.reconnectAttempts = 0;
    };

    this.ws.onmessage = (event) => {
      const data = JSON.parse(event.data);
      this.notifyListeners(data.type, data.payload);
    };

    this.ws.onclose = () => {
      console.log('WebSocket disconnected');
      this.reconnect();
    };

    this.ws.onerror = (error) => {
      console.error('WebSocket error:', error);
    };
  }

  disconnect() {
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
  }

  send(type, payload) {
    if (this.ws?.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify({ type, payload }));
    }
  }

  subscribe(event, callback) {
    if (!this.listeners.has(event)) {
      this.listeners.set(event, []);
    }
    this.listeners.get(event).push(callback);
  }

  unsubscribe(event, callback) {
    const callbacks = this.listeners.get(event);
    if (callbacks) {
      const index = callbacks.indexOf(callback);
      if (index > -1) {
        callbacks.splice(index, 1);
      }
    }
  }

  notifyListeners(event, data) {
    const callbacks = this.listeners.get(event);
    if (callbacks) {
      callbacks.forEach(callback => callback(data));
    }
  }

  reconnect() {
    if (this.reconnectAttempts < this.maxReconnectAttempts) {
      setTimeout(() => {
        this.reconnectAttempts++;
        this.connect();
      }, 1000 * Math.pow(2, this.reconnectAttempts));
    }
  }
}

export default WebSocketService;
```

## 🚁 **Three.js Integration**

### 3D Scene Setup
```javascript
// src/components/ThreeJSViewer.jsx
import * as THREE from 'three';

const initThreeJS = (containerRef) => {
  // Scene
  const scene = new THREE.Scene();
  scene.background = new THREE.Color(0x87CEEB);

  // Camera
  const camera = new THREE.PerspectiveCamera(
    75,
    window.innerWidth / window.innerHeight,
    0.1,
    1000
  );
  camera.position.set(15, 15, 15);

  // Renderer
  const renderer = new THREE.WebGLRenderer({ antialias: true });
  renderer.setSize(containerRef.current.clientWidth, containerRef.current.clientHeight);
  renderer.shadowMap.enabled = true;
  renderer.shadowMap.type = THREE.PCFSoftShadowMap;

  containerRef.current.appendChild(renderer.domElement);

  // Lighting
  const ambientLight = new THREE.AmbientLight(0x404040, 0.4);
  scene.add(ambientLight);

  const directionalLight = new THREE.DirectionalLight(0xffffff, 0.8);
  directionalLight.position.set(20, 20, 10);
  directionalLight.castShadow = true;
  scene.add(directionalLight);

  return { scene, camera, renderer };
};
```

### Drone Model Creation
```javascript
const createDroneModel = () => {
  const droneGroup = new THREE.Group();

  // Main body
  const bodyGeometry = new THREE.BoxGeometry(1.2, 0.3, 1.2);
  const bodyMaterial = new THREE.MeshLambertMaterial({ color: 0x2c3e50 });
  const body = new THREE.Mesh(bodyGeometry, bodyMaterial);
  body.castShadow = true;
  droneGroup.add(body);

  // Propellers
  const propPositions = [
    { x: 0.6, y: 0.4, z: 0.6 },
    { x: -0.6, y: 0.4, z: 0.6 },
    { x: 0.6, y: 0.4, z: -0.6 },
    { x: -0.6, y: 0.4, z: -0.6 }
  ];

  propPositions.forEach(pos => {
    const propGeometry = new THREE.CylinderGeometry(0.25, 0.25, 0.02);
    const propMaterial = new THREE.MeshLambertMaterial({ 
      color: 0x95a5a6,
      transparent: true,
      opacity: 0.7 
    });
    const propeller = new THREE.Mesh(propGeometry, propMaterial);
    propeller.position.set(pos.x, pos.y, pos.z);
    propeller.userData = { rotationSpeed: 0 };
    droneGroup.add(propeller);
  });

  return droneGroup;
};
```

## 📊 **Chart Integration**

### Recharts Configuration
```javascript
// src/components/MetricsChart.jsx
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer } from 'recharts';

const MetricsChart = ({ data }) => (
  <ResponsiveContainer width="100%" height={300}>
    <LineChart data={data}>
      <CartesianGrid strokeDasharray="3 3" />
      <XAxis dataKey="time" />
      <YAxis />
      <Tooltip />
      <Line 
        type="monotone" 
        dataKey="cpu" 
        stroke="#1976d2" 
        strokeWidth={2}
        dot={{ fill: '#1976d2', strokeWidth: 2, r: 4 }}
      />
      <Line 
        type="monotone" 
        dataKey="memory" 
        stroke="#dc004e" 
        strokeWidth={2}
        dot={{ fill: '#dc004e', strokeWidth: 2, r: 4 }}
      />
    </LineChart>
  </ResponsiveContainer>
);
```

## 🧪 **Testing Strategy**

### Unit Testing
```javascript
// src/components/__tests__/StatCard.test.jsx
import { render, screen } from '@testing-library/react';
import { ThemeProvider, createTheme } from '@mui/material/styles';
import StatCard from '../StatCard';

const theme = createTheme();

const renderWithTheme = (component) => {
  return render(
    <ThemeProvider theme={theme}>
      {component}
    </ThemeProvider>
  );
};

describe('StatCard', () => {
  test('renders title and value correctly', () => {
    renderWithTheme(
      <StatCard 
        title="Test Title" 
        value={42} 
        icon={<div>icon</div>}
        color="primary"
      />
    );
    
    expect(screen.getByText('Test Title')).toBeInTheDocument();
    expect(screen.getByText('42')).toBeInTheDocument();
  });

  test('shows loading state', () => {
    renderWithTheme(
      <StatCard 
        title="Test Title" 
        value={42} 
        loading={true}
        icon={<div>icon</div>}
        color="primary"
      />
    );
    
    expect(screen.getByRole('progressbar')).toBeInTheDocument();
  });
});
```

### E2E Testing
```javascript
// cypress/integration/dashboard.spec.js
describe('Dashboard', () => {
  beforeEach(() => {
    cy.visit('/');
  });

  it('should display overview dashboard by default', () => {
    cy.contains('Overview').should('be.visible');
    cy.get('[data-testid="stat-cards"]').should('exist');
  });

  it('should navigate to drone control', () => {
    cy.get('[data-testid="drone-nav"]').click();
    cy.contains('Drone 3D Visualization').should('be.visible');
  });

  it('should show device management table', () => {
    cy.get('[data-testid="devices-nav"]').click();
    cy.get('[data-testid="devices-table"]').should('exist');
  });
});
```

## 🔧 **Build & Deployment**

### Vite Configuration
```javascript
// vite.config.js
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

export default defineConfig({
  plugins: [react()],
  server: {
    port: 3000,
    host: true,
    open: true
  },
  build: {
    outDir: 'dist',
    sourcemap: true,
    rollupOptions: {
      output: {
        manualChunks: {
          vendor: ['react', 'react-dom'],
          mui: ['@mui/material', '@mui/icons-material'],
          charts: ['recharts'],
          three: ['three']
        }
      }
    }
  },
  optimizeDeps: {
    include: ['three']
  }
});
```

### Docker Configuration
```dockerfile
# Dockerfile
FROM node:18-alpine as builder

WORKDIR /app
COPY package*.json ./
RUN npm ci --only=production

COPY . .
RUN npm run build

FROM nginx:alpine
COPY --from=builder /app/dist /usr/share/nginx/html
COPY nginx.conf /etc/nginx/nginx.conf

EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
```

## 🚀 **Performance Optimization**

### Bundle Analysis
```bash
# Analyze bundle size
npm run build
npx vite-bundle-analyzer dist/stats.html
```

### Lazy Loading
```javascript
// src/App.jsx
import { lazy, Suspense } from 'react';

const DroneControlDashboard = lazy(() => import('./components/DroneControlDashboard'));
const DevicesDashboard = lazy(() => import('./components/DevicesDashboard'));

const App = () => (
  <Suspense fallback={<div>Loading...</div>}>
    {/* Component rendering */}
  </Suspense>
);
```

### Memoization
```javascript
// src/components/StatCard.jsx
import { memo } from 'react';

const StatCard = memo(({ title, value, loading, ...props }) => {
  // Component implementation
});

export default StatCard;
```

## 🔍 **Debugging**

### Browser DevTools
- **React DevTools** - Componenti e state
- **Three.js Inspector** - Scene 3D
- **Network Tab** - API calls
- **Performance Tab** - Performance analysis

### Console Debugging
```javascript
// Debug WebSocket messages
window.wsDebug = true;

// Debug Three.js scene
window.threeDebug = (scene) => {
  console.log('Scene objects:', scene.children);
  console.log('Scene stats:', {
    geometries: scene.children.filter(c => c.geometry).length,
    materials: scene.children.filter(c => c.material).length,
    lights: scene.children.filter(c => c.isLight).length
  });
};
```

## 📝 **Code Style**

### ESLint Configuration
```json
// .eslintrc.json
{
  "extends": [
    "react-app",
    "react-app/jest"
  ],
  "rules": {
    "indent": ["error", 2],
    "quotes": ["error", "single"],
    "semi": ["error", "always"],
    "no-unused-vars": "warn",
    "react/prop-types": "off"
  }
}
```

### Prettier Configuration
```json
// .prettierrc
{
  "singleQuote": true,
  "trailingComma": "es5",
  "tabWidth": 2,
  "semi": true,
  "printWidth": 80
}
```

## 🤝 **Contributing Workflow**

1. **Fork** il repository
2. **Create branch** da `develop`: `git checkout -b feature/my-feature`
3. **Develop** con test
4. **Commit** con messaggi descrittivi
5. **Push** e create **Pull Request**
6. **Code review** e merge

### Commit Messages
```
feat: add drone mission planning
fix: resolve WebSocket reconnection issue
docs: update API documentation
test: add unit tests for StatCard
refactor: optimize Three.js rendering
```

---

**Ultimo aggiornamento**: Gennaio 2025  
**Maintainer**: [@lucadag](https://github.com/lucadag)  
**Versione**: 2.0.0
