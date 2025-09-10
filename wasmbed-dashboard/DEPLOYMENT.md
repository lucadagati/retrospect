# 🚀 Deployment Guide - Wasmbed Dashboard

Guida per il deployment della dashboard Wasmbed in diversi ambienti.

## 📋 **Overview**

La dashboard può essere deployata in diversi modi:
- **Development**: Server locale per sviluppo
- **Production**: Server web statico (Nginx, Apache)
- **Docker**: Containerizzazione per deployment scalabile
- **Kubernetes**: Deployment su cluster K8s
- **Cloud**: AWS S3, Netlify, Vercel

## 🛠️ **Development Deployment**

### Local Development
```bash
# Clone repository
git clone https://github.com/yourusername/wasmbed-dashboard.git
cd wasmbed-dashboard

# Install dependencies
npm install

# Start development server
npm run dev
```

**URL**: http://localhost:3000  
**Hot Reload**: ✅ Abilitato  
**Source Maps**: ✅ Abilitato

### Environment Variables
```bash
# .env.local
VITE_API_BASE_URL=http://localhost:8080
VITE_WEBSOCKET_URL=ws://localhost:8080/ws
VITE_ENABLE_MOCK_DATA=true
VITE_LOG_LEVEL=debug
```

## 📦 **Production Build**

### Build Static Files
```bash
# Build for production
npm run build

# Preview production build
npm run preview
```

**Output**: `dist/` directory  
**Size**: ~5MB (ottimizzato)  
**Chunks**: Vendor, MUI, Charts, Three.js separati

### Build Optimization
```javascript
// vite.config.js
export default defineConfig({
  build: {
    outDir: 'dist',
    sourcemap: false, // Disable in production
    minify: 'terser',
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
  }
});
```

## 🐳 **Docker Deployment**

### Dockerfile Multi-stage
```dockerfile
# Build stage
FROM node:18-alpine as builder

WORKDIR /app
COPY package*.json ./
RUN npm ci --only=production && npm cache clean --force

COPY . .
RUN npm run build

# Production stage
FROM nginx:alpine

# Copy built files
COPY --from=builder /app/dist /usr/share/nginx/html

# Copy nginx configuration
COPY nginx.conf /etc/nginx/nginx.conf

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD curl -f http://localhost/ || exit 1

EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
```

### Nginx Configuration
```nginx
# nginx.conf
server {
    listen 80;
    server_name localhost;
    root /usr/share/nginx/html;
    index index.html;

    # Gzip compression
    gzip on;
    gzip_vary on;
    gzip_min_length 1024;
    gzip_types text/plain text/css text/xml text/javascript application/javascript application/xml+rss application/json;

    # Security headers
    add_header X-Frame-Options "SAMEORIGIN" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-XSS-Protection "1; mode=block" always;

    # Static files caching
    location ~* \.(js|css|png|jpg|jpeg|gif|ico|svg)$ {
        expires 1y;
        add_header Cache-Control "public, immutable";
    }

    # API proxy
    location /api/ {
        proxy_pass http://wasmbed-gateway:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    # WebSocket proxy
    location /ws {
        proxy_pass http://wasmbed-gateway:8080;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
    }

    # SPA routing
    location / {
        try_files $uri $uri/ /index.html;
    }
}
```

### Docker Commands
```bash
# Build image
docker build -t wasmbed-dashboard:latest .

# Run container
docker run -d \
  --name wasmbed-dashboard \
  -p 3000:80 \
  -e VITE_API_BASE_URL=http://your-gateway:8080 \
  wasmbed-dashboard:latest

# Docker Compose
version: '3.8'
services:
  dashboard:
    build: .
    ports:
      - "3000:80"
    environment:
      - VITE_API_BASE_URL=http://gateway:8080
    depends_on:
      - gateway
```

## ☸️ **Kubernetes Deployment**

### Deployment YAML
```yaml
# k8s/deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: wasmbed-dashboard
  labels:
    app: wasmbed-dashboard
spec:
  replicas: 3
  selector:
    matchLabels:
      app: wasmbed-dashboard
  template:
    metadata:
      labels:
        app: wasmbed-dashboard
    spec:
      containers:
      - name: dashboard
        image: wasmbed-dashboard:latest
        ports:
        - containerPort: 80
        env:
        - name: VITE_API_BASE_URL
          value: "http://wasmbed-gateway:8080"
        resources:
          requests:
            memory: "128Mi"
            cpu: "100m"
          limits:
            memory: "256Mi"
            cpu: "200m"
        livenessProbe:
          httpGet:
            path: /
            port: 80
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /
            port: 80
          initialDelaySeconds: 5
          periodSeconds: 5
---
apiVersion: v1
kind: Service
metadata:
  name: wasmbed-dashboard-service
spec:
  selector:
    app: wasmbed-dashboard
  ports:
    - protocol: TCP
      port: 80
      targetPort: 80
  type: ClusterIP
---
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: wasmbed-dashboard-ingress
  annotations:
    nginx.ingress.kubernetes.io/rewrite-target: /
    cert-manager.io/cluster-issuer: "letsencrypt-prod"
spec:
  tls:
  - hosts:
    - dashboard.wasmbed.io
    secretName: dashboard-tls
  rules:
  - host: dashboard.wasmbed.io
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: wasmbed-dashboard-service
            port:
              number: 80
```

### ConfigMap per Environment
```yaml
# k8s/configmap.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: dashboard-config
data:
  VITE_API_BASE_URL: "https://api.wasmbed.io"
  VITE_WEBSOCKET_URL: "wss://api.wasmbed.io/ws"
  VITE_ENABLE_MOCK_DATA: "false"
  VITE_LOG_LEVEL: "info"
```

### Deploy Commands
```bash
# Apply configurations
kubectl apply -f k8s/configmap.yaml
kubectl apply -f k8s/deployment.yaml

# Check status
kubectl get pods -l app=wasmbed-dashboard
kubectl get services wasmbed-dashboard-service
kubectl get ingress wasmbed-dashboard-ingress

# Logs
kubectl logs -l app=wasmbed-dashboard -f
```

## ☁️ **Cloud Deployment**

### AWS S3 + CloudFront
```bash
# Build and sync to S3
npm run build
aws s3 sync dist/ s3://wasmbed-dashboard-bucket --delete

# Invalidate CloudFront
aws cloudfront create-invalidation \
  --distribution-id E1234567890 \
  --paths "/*"
```

### Netlify
```bash
# netlify.toml
[build]
  publish = "dist"
  command = "npm run build"

[build.environment]
  VITE_API_BASE_URL = "https://api.wasmbed.io"

[[redirects]]
  from = "/*"
  to = "/index.html"
  status = 200
```

### Vercel
```json
// vercel.json
{
  "builds": [
    {
      "src": "package.json",
      "use": "@vercel/static-build",
      "config": {
        "distDir": "dist"
      }
    }
  ],
  "routes": [
    {
      "src": "/(.*)",
      "dest": "/index.html"
    }
  ],
  "env": {
    "VITE_API_BASE_URL": "https://api.wasmbed.io"
  }
}
```

## 🔧 **Production Configuration**

### Environment Variables
```bash
# Production .env
VITE_API_BASE_URL=https://api.wasmbed.io
VITE_WEBSOCKET_URL=wss://api.wasmbed.io/ws
VITE_ENABLE_MOCK_DATA=false
VITE_LOG_LEVEL=warn
VITE_SENTRY_DSN=https://your-sentry-dsn
```

### Security Headers
```nginx
# Additional security headers
add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;
add_header Content-Security-Policy "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; connect-src 'self' wss:; font-src 'self';" always;
add_header Referrer-Policy "strict-origin-when-cross-origin" always;
```

### Performance Optimization
```nginx
# Enable HTTP/2
listen 443 ssl http2;

# Enable Brotli compression (if available)
brotli on;
brotli_comp_level 6;
brotli_types text/plain text/css application/json application/javascript text/xml application/xml text/javascript;

# Static file optimization
location ~* \.(js|css)$ {
    expires 1y;
    add_header Cache-Control "public, immutable";
    add_header Vary "Accept-Encoding";
}
```

## 📊 **Monitoring & Logging**

### Health Checks
```javascript
// src/utils/healthCheck.js
export const healthCheck = async () => {
  try {
    const response = await fetch('/api/health');
    return response.ok;
  } catch (error) {
    console.error('Health check failed:', error);
    return false;
  }
};
```

### Error Tracking
```javascript
// src/utils/sentry.js
import * as Sentry from "@sentry/react";

Sentry.init({
  dsn: import.meta.env.VITE_SENTRY_DSN,
  environment: import.meta.env.MODE,
  integrations: [
    new Sentry.BrowserTracing(),
  ],
  tracesSampleRate: 1.0,
});
```

### Prometheus Metrics
```nginx
# Nginx metrics for Prometheus
location /metrics {
    stub_status on;
    access_log off;
    allow 10.0.0.0/8;
    deny all;
}
```

## 🔄 **CI/CD Pipeline**

### GitHub Actions
```yaml
# .github/workflows/deploy.yml
name: Deploy Dashboard

on:
  push:
    branches: [main]

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3
    
    - name: Setup Node.js
      uses: actions/setup-node@v3
      with:
        node-version: '18'
        cache: 'npm'
    
    - name: Install dependencies
      run: npm ci
    
    - name: Run tests
      run: npm test
    
    - name: Build
      run: npm run build
      env:
        VITE_API_BASE_URL: ${{ secrets.API_BASE_URL }}
    
    - name: Build Docker image
      run: docker build -t wasmbed-dashboard:${{ github.sha }} .
    
    - name: Push to registry
      run: |
        docker tag wasmbed-dashboard:${{ github.sha }} registry.wasmbed.io/dashboard:latest
        docker push registry.wasmbed.io/dashboard:latest
    
    - name: Deploy to Kubernetes
      run: |
        kubectl set image deployment/wasmbed-dashboard dashboard=registry.wasmbed.io/dashboard:latest
```

## 🚨 **Troubleshooting**

### Common Issues

**1. Build Fails**
```bash
# Clear cache
rm -rf node_modules package-lock.json
npm install

# Check Node.js version
node --version  # Should be 18+
```

**2. WebSocket Connection Fails**
```javascript
// Check WebSocket URL
console.log('WebSocket URL:', import.meta.env.VITE_WEBSOCKET_URL);

// Test connection manually
const ws = new WebSocket('ws://your-api/ws');
ws.onopen = () => console.log('Connected');
ws.onerror = (error) => console.error('WebSocket error:', error);
```

**3. API Calls Fail**
```nginx
# Check CORS headers
add_header Access-Control-Allow-Origin "https://dashboard.wasmbed.io" always;
add_header Access-Control-Allow-Methods "GET, POST, PUT, DELETE, OPTIONS" always;
add_header Access-Control-Allow-Headers "Content-Type, Authorization" always;
```

**4. 3D Visualization Issues**
```javascript
// Check WebGL support
const canvas = document.createElement('canvas');
const gl = canvas.getContext('webgl') || canvas.getContext('experimental-webgl');
console.log('WebGL supported:', !!gl);
```

### Performance Issues
```bash
# Analyze bundle size
npm run build
npx vite-bundle-analyzer dist/stats.html

# Check memory usage
# Chrome DevTools > Performance > Memory
```

---

**Ultimo aggiornamento**: Gennaio 2025  
**Maintainer**: [@lucadag](https://github.com/lucadag)  
**Versione**: 2.0.0
