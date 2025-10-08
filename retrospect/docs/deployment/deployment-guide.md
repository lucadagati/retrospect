# Deployment Guide

## Overview

This document provides comprehensive deployment instructions for the Wasmbed platform, including production deployment, configuration management, and scaling strategies.

## Deployment Architecture

### Production Architecture

**High Availability Setup**:
- Multiple gateway instances with load balancing
- Kubernetes cluster with multiple nodes
- Database clustering and replication
- Network load balancers
- Health checks and monitoring

**Scalability Considerations**:
- Horizontal scaling of gateway instances
- Auto-scaling based on load metrics
- Resource quotas and limits
- Performance monitoring
- Capacity planning

## Prerequisites

### System Requirements

**Minimum Production Requirements**:
- CPU: 8 cores per node
- RAM: 16GB per node
- Storage: 100GB SSD per node
- Network: 1Gbps bandwidth

**Recommended Production Requirements**:
- CPU: 16 cores per node
- RAM: 32GB per node
- Storage: 500GB NVMe SSD per node
- Network: 10Gbps bandwidth

### Software Dependencies

**Kubernetes Cluster**:
- Kubernetes 1.25+
- etcd 3.5+
- CNI plugin (Calico recommended)
- Ingress controller (NGINX recommended)
- Storage class (local-path or cloud storage)

**Additional Software**:
- Docker 20.10+
- Helm 3.0+
- kubectl 1.25+
- QEMU system emulators
- Monitoring stack (Prometheus, Grafana)

## Deployment Methods

### Method 1: Automated Deployment

**Complete Platform Deployment**:
```bash
# Deploy complete platform
./scripts/deploy.sh

# Verify deployment
./scripts/monitor.sh status
```

**Deployment Process**:
1. Create Kubernetes cluster
2. Build and push Docker images
3. Generate TLS certificates
4. Deploy Kubernetes resources
5. Verify deployment status

### Method 2: Manual Deployment

#### Step 1: Create Kubernetes Cluster

**k3d Cluster (Development)**:
```bash
# Create k3d cluster
k3d cluster create wasmbed \
  --port "8080:80@loadbalancer" \
  --port "4423:443@loadbalancer" \
  --agents 3 \
  --k3s-arg "--disable=traefik@server:0"

# Verify cluster
kubectl cluster-info
kubectl get nodes
```

**Production Cluster (Cloud)**:
```bash
# Create production cluster (example for AWS EKS)
eksctl create cluster \
  --name wasmbed-prod \
  --version 1.25 \
  --region us-west-2 \
  --nodegroup-name workers \
  --node-type m5.xlarge \
  --nodes 3 \
  --nodes-min 1 \
  --nodes-max 10 \
  --managed
```

#### Step 2: Build and Push Docker Images

**Build Images**:
```bash
# Build gateway image
docker build -f Dockerfile.gateway -t wasmbed-gateway:latest .

# Build controller image
docker build -f Dockerfile.controller -t wasmbed-k8s-controller:latest .

# Build QEMU bridge image
docker build -f Dockerfile.qemu-bridge -t wasmbed-qemu-bridge:latest .
```

**Push Images**:
```bash
# Tag images for registry
docker tag wasmbed-gateway:latest registry.example.com/wasmbed-gateway:latest
docker tag wasmbed-k8s-controller:latest registry.example.com/wasmbed-k8s-controller:latest
docker tag wasmbed-qemu-bridge:latest registry.example.com/wasmbed-qemu-bridge:latest

# Push images
docker push registry.example.com/wasmbed-gateway:latest
docker push registry.example.com/wasmbed-k8s-controller:latest
docker push registry.example.com/wasmbed-qemu-bridge:latest
```

#### Step 3: Generate TLS Certificates

**Certificate Generation**:
```bash
# Generate CA certificate
openssl genrsa -out ca-key.pem 4096
openssl req -new -x509 -days 365 -key ca-key.pem -out ca-cert.pem

# Generate server certificate
openssl genrsa -out server-key.pem 4096
openssl req -new -key server-key.pem -out server.csr
openssl x509 -req -days 365 -in server.csr -CA ca-cert.pem -CAkey ca-key.pem -out server-cert.pem

# Generate client certificate
openssl genrsa -out client-key.pem 4096
openssl req -new -key client-key.pem -out client.csr
openssl x509 -req -days 365 -in client.csr -CA ca-cert.pem -CAkey ca-key.pem -out client-cert.pem
```

**Create Kubernetes Secrets**:
```bash
# Create TLS secret
kubectl create secret tls wasmbed-tls-secret-rsa \
  --cert=server-cert.pem \
  --key=server-key.pem \
  -n wasmbed

# Create CA secret
kubectl create secret generic wasmbed-ca-secret-rsa \
  --from-file=ca-cert.pem \
  -n wasmbed
```

#### Step 4: Deploy Kubernetes Resources

**Deploy CRDs**:
```bash
# Deploy Custom Resource Definitions
kubectl apply -f resources/k8s/crds/application-crd.yaml
kubectl apply -f resources/k8s/crds/device-crd.yaml
```

**Deploy RBAC**:
```bash
# Deploy RBAC resources
kubectl apply -f resources/k8s/rbac/controller-rbac.yaml
kubectl apply -f resources/k8s/rbac/gateway-rbac.yaml
```

**Deploy Gateway**:
```bash
# Deploy gateway
kubectl apply -f resources/k8s/gateway-configmap.yaml
kubectl apply -f resources/k8s/gateway-service.yaml
kubectl apply -f resources/k8s/111-statefulset-gateway.yaml
```

**Deploy Controller**:
```bash
# Deploy controller
kubectl apply -f resources/k8s/controller-deployment.yaml
```

#### Step 5: Verify Deployment

**Check Deployment Status**:
```bash
# Check pods
kubectl get pods -n wasmbed

# Check services
kubectl get services -n wasmbed

# Check CRDs
kubectl get crds | grep wasmbed

# Check logs
kubectl logs -f deployment/wasmbed-gateway -n wasmbed
kubectl logs -f deployment/wasmbed-k8s-controller -n wasmbed
```

### Method 3: Helm Deployment

**Helm Chart Structure**:
```
wasmbed-platform/
├── Chart.yaml
├── values.yaml
├── templates/
│   ├── crds/
│   ├── rbac/
│   ├── gateway/
│   ├── controller/
│   └── monitoring/
└── charts/
```

**Deploy with Helm**:
```bash
# Add Helm repository
helm repo add wasmbed https://charts.wasmbed.io
helm repo update

# Install platform
helm install wasmbed-platform wasmbed/wasmbed-platform \
  --namespace wasmbed \
  --create-namespace \
  --values values.yaml

# Upgrade platform
helm upgrade wasmbed-platform wasmbed/wasmbed-platform \
  --namespace wasmbed \
  --values values.yaml
```

## Configuration Management

### Environment Configuration

**Gateway Configuration**:
```yaml
# Gateway ConfigMap
apiVersion: v1
kind: ConfigMap
metadata:
  name: wasmbed-gateway-config
  namespace: wasmbed
data:
  pairing_mode: "false"
  pairing_timeout_seconds: "300"
  heartbeat_timeout_seconds: "30"
  max_devices: "1000"
  log_level: "info"
  max_connections: "10000"
  connection_timeout: "30"
  read_timeout: "60"
  write_timeout: "60"
```

**Controller Configuration**:
```yaml
# Controller ConfigMap
apiVersion: v1
kind: ConfigMap
metadata:
  name: wasmbed-controller-config
  namespace: wasmbed
data:
  reconciliation_interval: "30s"
  max_retries: "3"
  retry_delay: "5s"
  log_level: "info"
  metrics_enabled: "true"
  health_check_interval: "10s"
```

### Resource Limits and Quotas

**Resource Limits**:
```yaml
# Gateway resource limits
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: wasmbed-gateway
spec:
  template:
    spec:
      containers:
      - name: wasmbed-gateway
        resources:
          requests:
            cpu: "500m"
            memory: "1Gi"
          limits:
            cpu: "2000m"
            memory: "4Gi"
```

**Namespace Quotas**:
```yaml
# Resource quota
apiVersion: v1
kind: ResourceQuota
metadata:
  name: wasmbed-quota
  namespace: wasmbed
spec:
  hard:
    requests.cpu: "10"
    requests.memory: "20Gi"
    limits.cpu: "20"
    limits.memory: "40Gi"
    pods: "50"
    services: "10"
    persistentvolumeclaims: "10"
```

### Security Configuration

**Network Policies**:
```yaml
# Network policy
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: wasmbed-network-policy
  namespace: wasmbed
spec:
  podSelector: {}
  policyTypes:
  - Ingress
  - Egress
  ingress:
  - from:
    - namespaceSelector:
        matchLabels:
          name: wasmbed
    ports:
    - protocol: TCP
      port: 8080
    - protocol: TCP
      port: 4423
  egress:
  - to:
    - namespaceSelector:
        matchLabels:
          name: kube-system
    ports:
    - protocol: TCP
      port: 443
```

**Pod Security Policies**:
```yaml
# Pod security policy
apiVersion: policy/v1beta1
kind: PodSecurityPolicy
metadata:
  name: wasmbed-psp
spec:
  privileged: false
  allowPrivilegeEscalation: false
  requiredDropCapabilities:
  - ALL
  volumes:
  - 'configMap'
  - 'emptyDir'
  - 'projected'
  - 'secret'
  - 'downwardAPI'
  - 'persistentVolumeClaim'
  runAsUser:
    rule: 'MustRunAsNonRoot'
  seLinux:
    rule: 'RunAsAny'
  fsGroup:
    rule: 'RunAsAny'
```

## Scaling Strategies

### Horizontal Scaling

**Gateway Scaling**:
```yaml
# Horizontal Pod Autoscaler
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: wasmbed-gateway-hpa
  namespace: wasmbed
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: StatefulSet
    name: wasmbed-gateway
  minReplicas: 3
  maxReplicas: 10
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
```

**Cluster Scaling**:
```bash
# Scale cluster nodes
eksctl scale nodegroup \
  --cluster wasmbed-prod \
  --name workers \
  --nodes 5 \
  --nodes-min 3 \
  --nodes-max 15
```

### Vertical Scaling

**Resource Optimization**:
```yaml
# Vertical Pod Autoscaler
apiVersion: autoscaling.k8s.io/v1
kind: VerticalPodAutoscaler
metadata:
  name: wasmbed-gateway-vpa
  namespace: wasmbed
spec:
  targetRef:
    apiVersion: apps/v1
    kind: StatefulSet
    name: wasmbed-gateway
  updatePolicy:
    updateMode: "Auto"
  resourcePolicy:
    containerPolicies:
    - containerName: wasmbed-gateway
      minAllowed:
        cpu: "100m"
        memory: "256Mi"
      maxAllowed:
        cpu: "4000m"
        memory: "8Gi"
```

## Monitoring and Observability

### Prometheus Monitoring

**Prometheus Configuration**:
```yaml
# Prometheus ConfigMap
apiVersion: v1
kind: ConfigMap
metadata:
  name: prometheus-config
  namespace: monitoring
data:
  prometheus.yml: |
    global:
      scrape_interval: 15s
    scrape_configs:
    - job_name: 'wasmbed-gateway'
      static_configs:
      - targets: ['wasmbed-gateway-service:8080']
      metrics_path: '/metrics'
    - job_name: 'wasmbed-controller'
      static_configs:
      - targets: ['wasmbed-controller-service:8080']
      metrics_path: '/metrics'
```

**ServiceMonitor**:
```yaml
# ServiceMonitor for Prometheus Operator
apiVersion: monitoring.coreos.com/v1
kind: ServiceMonitor
metadata:
  name: wasmbed-gateway-monitor
  namespace: monitoring
spec:
  selector:
    matchLabels:
      app: wasmbed-gateway
  endpoints:
  - port: metrics
    path: /metrics
    interval: 30s
```

### Grafana Dashboards

**Dashboard Configuration**:
```yaml
# Grafana Dashboard ConfigMap
apiVersion: v1
kind: ConfigMap
metadata:
  name: wasmbed-dashboard
  namespace: monitoring
data:
  dashboard.json: |
    {
      "dashboard": {
        "title": "Wasmbed Platform Dashboard",
        "panels": [
          {
            "title": "Gateway CPU Usage",
            "type": "graph",
            "targets": [
              {
                "expr": "rate(process_cpu_seconds_total{job=\"wasmbed-gateway\"}[5m])",
                "legendFormat": "CPU Usage"
              }
            ]
          }
        ]
      }
    }
```

### Logging

**Fluentd Configuration**:
```yaml
# Fluentd ConfigMap
apiVersion: v1
kind: ConfigMap
metadata:
  name: fluentd-config
  namespace: logging
data:
  fluent.conf: |
    <source>
      @type tail
      path /var/log/containers/*wasmbed*.log
      pos_file /var/log/fluentd-containers.log.pos
      tag kubernetes.*
      format json
    </source>
    <match kubernetes.**>
      @type elasticsearch
      host elasticsearch.logging.svc.cluster.local
      port 9200
      index_name wasmbed-logs
    </match>
```

## Backup and Recovery

### Data Backup

**etcd Backup**:
```bash
# Backup etcd
ETCDCTL_API=3 etcdctl snapshot save backup.db \
  --endpoints=https://127.0.0.1:2379 \
  --cacert=/etc/kubernetes/pki/etcd/ca.crt \
  --cert=/etc/kubernetes/pki/etcd/server.crt \
  --key=/etc/kubernetes/pki/etcd/server.key
```

**Application Data Backup**:
```bash
# Backup application data
kubectl get applications -n wasmbed -o yaml > applications-backup.yaml
kubectl get devices -n wasmbed -o yaml > devices-backup.yaml
kubectl get configmaps -n wasmbed -o yaml > configmaps-backup.yaml
kubectl get secrets -n wasmbed -o yaml > secrets-backup.yaml
```

### Disaster Recovery

**Recovery Procedures**:
```bash
# Restore etcd
ETCDCTL_API=3 etcdctl snapshot restore backup.db \
  --data-dir=/var/lib/etcd-restore \
  --name=etcd-restore \
  --initial-cluster=etcd-restore=https://127.0.0.1:2380 \
  --initial-cluster-token=etcd-cluster-1 \
  --initial-advertise-peer-urls=https://127.0.0.1:2380

# Restore application data
kubectl apply -f applications-backup.yaml
kubectl apply -f devices-backup.yaml
kubectl apply -f configmaps-backup.yaml
kubectl apply -f secrets-backup.yaml
```

## Security Hardening

### Network Security

**Ingress Configuration**:
```yaml
# NGINX Ingress
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: wasmbed-ingress
  namespace: wasmbed
  annotations:
    nginx.ingress.kubernetes.io/ssl-redirect: "true"
    nginx.ingress.kubernetes.io/force-ssl-redirect: "true"
    nginx.ingress.kubernetes.io/rate-limit: "100"
    nginx.ingress.kubernetes.io/rate-limit-window: "1m"
spec:
  tls:
  - hosts:
    - wasmbed.example.com
    secretName: wasmbed-tls-secret
  rules:
  - host: wasmbed.example.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: wasmbed-gateway-service
            port:
              number: 8080
```

### Access Control

**RBAC Configuration**:
```yaml
# ClusterRole for admin
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRole
metadata:
  name: wasmbed-admin
rules:
- apiGroups: ["wasmbed.github.io"]
  resources: ["applications", "devices"]
  verbs: ["get", "list", "watch", "create", "update", "patch", "delete"]
- apiGroups: [""]
  resources: ["pods", "services", "configmaps", "secrets"]
  verbs: ["get", "list", "watch", "create", "update", "patch", "delete"]
```

## Performance Optimization

### Resource Optimization

**CPU Optimization**:
```yaml
# CPU limits and requests
resources:
  requests:
    cpu: "500m"
  limits:
    cpu: "2000m"
```

**Memory Optimization**:
```yaml
# Memory limits and requests
resources:
  requests:
    memory: "1Gi"
  limits:
    memory: "4Gi"
```

### Network Optimization

**Connection Pooling**:
```yaml
# Connection pool configuration
env:
- name: MAX_CONNECTIONS
  value: "10000"
- name: CONNECTION_TIMEOUT
  value: "30s"
- name: READ_TIMEOUT
  value: "60s"
- name: WRITE_TIMEOUT
  value: "60s"
```

## Maintenance

### Regular Maintenance

**Update Procedures**:
```bash
# Update platform
helm upgrade wasmbed-platform wasmbed/wasmbed-platform \
  --namespace wasmbed \
  --values values.yaml \
  --version 1.1.0

# Rollback if needed
helm rollback wasmbed-platform 1 --namespace wasmbed
```

**Health Checks**:
```bash
# Check platform health
./scripts/monitor.sh health

# Check component health
kubectl get pods -n wasmbed
kubectl get services -n wasmbed
kubectl get ingress -n wasmbed
```

### Troubleshooting

**Common Issues**:
- Pod startup failures
- Service connectivity issues
- Resource exhaustion
- Network policy conflicts
- Certificate expiration

**Debugging Commands**:
```bash
# Check pod logs
kubectl logs -f deployment/wasmbed-gateway -n wasmbed

# Check pod status
kubectl describe pod <pod-name> -n wasmbed

# Check service endpoints
kubectl get endpoints -n wasmbed

# Check network policies
kubectl get networkpolicies -n wasmbed
```
