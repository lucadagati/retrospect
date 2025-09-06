# Phase 11: Production Hardening

## 🛡️ **Production Security & Hardening Guide**

### Overview
This phase focuses on hardening the Wasmbed platform for production deployment, implementing enterprise-grade security, high availability, and operational best practices.

##  **Security Hardening**

### 1. **Certificate Management**

#### Production Certificate Authority
```bash
# Generate production CA with proper key size
openssl genrsa -out production-ca.key 4096
openssl req -new -x509 -days 3650 -key production-ca.key -out production-ca.crt \
  -subj "/C=IT/ST=Italy/L=Milan/O=Wasmbed/OU=Production/CN=Wasmbed Production CA"

# Generate server certificates with proper SAN
openssl req -newkey rsa:2048 -keyout production-server.key -out production-server.csr \
  -subj "/C=IT/ST=Italy/L=Milan/O=Wasmbed/OU=Production/CN=wasmbed-gateway.wasmbed.svc.cluster.local" \
  -addext "subjectAltName=DNS:wasmbed-gateway.wasmbed.svc.cluster.local,DNS:*.wasmbed.svc.cluster.local,IP:10.43.218.8"

openssl x509 -req -in production-server.csr -CA production-ca.crt -CAkey production-ca.key \
  -CAcreateserial -out production-server.crt -days 365 -extensions v3_req
```

#### Certificate Rotation
```yaml
# Certificate rotation job
apiVersion: batch/v1
kind: CronJob
metadata:
  name: certificate-rotation
  namespace: wasmbed
spec:
  schedule: "0 2 * * 0"  # Weekly at 2 AM Sunday
  jobTemplate:
    spec:
      template:
        spec:
          containers:
          - name: cert-rotation
            image: wasmbed-cert-tool:latest
            command: ["/app/rotate-certificates"]
            env:
            - name: CA_KEY_PATH
              value: "/etc/certs/ca.key"
            - name: CA_CERT_PATH
              value: "/etc/certs/ca.crt"
          volumes:
          - name: certs
            secret:
              secretName: wasmbed-production-certs
```

### 2. **Network Security**

#### Network Policies
```yaml
# Restrict pod-to-pod communication
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: wasmbed-network-policy
  namespace: wasmbed
spec:
  podSelector:
    matchLabels:
      app: wasmbed-gateway
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
      port: 4423
  egress:
  - to:
    - namespaceSelector:
        matchLabels:
          name: kube-system
    ports:
    - protocol: TCP
      port: 443
  - to:
    - namespaceSelector:
        matchLabels:
          name: wasmbed
    ports:
    - protocol: TCP
      port: 8080
```

#### TLS Configuration Hardening
```rust
// Enhanced TLS configuration
use rustls::{ServerConfig, PrivateKey, Certificate};
use rustls::server::AllowAnyAuthenticatedClient;

pub fn create_production_tls_config() -> Result<ServerConfig, Box<dyn std::error::Error>> {
    let mut config = ServerConfig::builder()
        .with_safe_defaults()
        .with_client_cert_verifier(AllowAnyAuthenticatedClient::new(
            load_ca_certificates("production-ca.crt")?
        ))
        .with_single_cert(
            load_certificates("production-server.crt")?,
            load_private_key("production-server.key")?
        )?;
    
    // Hardened cipher suites
    config.cipher_suites = vec![
        &rustls::cipher_suite::TLS13_AES_256_GCM_SHA384,
        &rustls::cipher_suite::TLS13_CHACHA20_POLY1305_SHA256,
    ];
    
    // Strict security settings
    config.max_fragment_size = Some(16384);
    config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
    
    Ok(config)
}
```

### 3. **RBAC Hardening**

#### Enhanced RBAC Configuration
```yaml
# Production RBAC with minimal privileges
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRole
metadata:
  name: wasmbed-controller-production
rules:
- apiGroups: ["wasmbed.github.io"]
  resources: ["devices", "applications"]
  verbs: ["get", "list", "watch", "create", "update", "patch", "delete"]
- apiGroups: [""]
  resources: ["pods", "services", "events"]
  verbs: ["get", "list", "watch", "create", "update", "patch"]
- apiGroups: [""]
  resources: ["configmaps", "secrets"]
  verbs: ["get", "list", "watch"]
- apiGroups: ["coordination.k8s.io"]
  resources: ["leases"]
  verbs: ["get", "list", "watch", "create", "update", "patch", "delete"]
---
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRoleBinding
metadata:
  name: wasmbed-controller-production
subjects:
- kind: ServiceAccount
  name: wasmbed-controller-sa
  namespace: wasmbed
roleRef:
  kind: ClusterRole
  name: wasmbed-controller-production
  apiGroup: rbac.authorization.k8s.io
```

##  **High Availability Setup**

### 1. **Multi-Zone Deployment**

#### Gateway StatefulSet with Anti-Affinity
```yaml
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: wasmbed-gateway-ha
  namespace: wasmbed
spec:
  serviceName: wasmbed-gateway-service
  replicas: 5  # Increased for HA
  selector:
    matchLabels:
      app: wasmbed-gateway
  template:
    metadata:
      labels:
        app: wasmbed-gateway
    spec:
      serviceAccountName: wasmbed-gateway
      # Anti-affinity for high availability
      affinity:
        podAntiAffinity:
          requiredDuringSchedulingIgnoredDuringExecution:
          - labelSelector:
              matchExpressions:
              - key: app
                operator: In
                values:
                - wasmbed-gateway
            topologyKey: kubernetes.io/hostname
        podAffinity:
          preferredDuringSchedulingIgnoredDuringExecution:
          - weight: 100
            podAffinityTerm:
              labelSelector:
                matchExpressions:
                - key: app
                  operator: In
                  values:
                  - wasmbed-gateway
              topologyKey: topology.kubernetes.io/zone
      containers:
      - name: wasmbed-gateway
        image: wasmbed-gateway:latest
        ports:
        - containerPort: 4423
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "512Mi"
            cpu: "500m"
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
```

### 2. **Load Balancing**

#### Service with Load Balancer
```yaml
apiVersion: v1
kind: Service
metadata:
  name: wasmbed-gateway-lb
  namespace: wasmbed
  annotations:
    service.beta.kubernetes.io/aws-load-balancer-type: nlb
    service.beta.kubernetes.io/aws-load-balancer-cross-zone-load-balancing-enabled: "true"
    service.beta.kubernetes.io/aws-load-balancer-health-check-path: "/health"
spec:
  type: LoadBalancer
  ports:
  - port: 4423
    targetPort: 4423
    protocol: TCP
    name: wasmbed-gateway
  selector:
    app: wasmbed-gateway
  sessionAffinity: ClientIP
  sessionAffinityConfig:
    clientIP:
      timeoutSeconds: 3600
```

### 3. **Controller High Availability**

#### Controller Deployment with Leader Election
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: wasmbed-k8s-controller-ha
  namespace: wasmbed
spec:
  replicas: 3  # Multiple replicas for HA
  selector:
    matchLabels:
      app: wasmbed-k8s-controller
  template:
    metadata:
      labels:
        app: wasmbed-k8s-controller
    spec:
      serviceAccountName: wasmbed-controller-sa
      containers:
      - name: controller
        image: wasmbed-k8s-controller:latest
        env:
        - name: LEADER_ELECTION_ENABLED
          value: "true"
        - name: LEADER_ELECTION_NAMESPACE
          value: "wasmbed"
        - name: LEADER_ELECTION_ID
          value: "wasmbed-controller"
        resources:
          requests:
            memory: "128Mi"
            cpu: "200m"
          limits:
            memory: "256Mi"
            cpu: "400m"
```

##  **Backup and Recovery**

### 1. **Data Backup Strategy**

#### Velero Backup Configuration
```yaml
apiVersion: velero.io/v1
kind: Schedule
metadata:
  name: wasmbed-daily-backup
  namespace: velero
spec:
  schedule: "0 1 * * *"  # Daily at 1 AM
  template:
    includedNamespaces:
    - wasmbed
    includedResources:
    - devices.wasmbed.github.io
    - applications.wasmbed.github.io
    - secrets
    - configmaps
    storageLocation: default
    volumeSnapshotLocations:
    - default
---
apiVersion: velero.io/v1
kind: BackupStorageLocation
metadata:
  name: default
  namespace: velero
spec:
  provider: aws
  objectStorage:
    bucket: wasmbed-backups
  config:
    region: eu-west-1
```

### 2. **Disaster Recovery Plan**

#### Recovery Procedures
```bash
#!/bin/bash
# Disaster recovery script
set -e

echo "🚨 Starting Wasmbed disaster recovery..."

# 1. Restore from latest backup
velero restore create --from-backup wasmbed-daily-backup-$(date +%Y%m%d)

# 2. Verify cluster health
kubectl get nodes
kubectl get pods -n wasmbed

# 3. Restore certificates
kubectl create secret generic wasmbed-production-certs \
  --from-file=ca.crt=production-ca.crt \
  --from-file=server.crt=production-server.crt \
  --from-file=server.key=production-server.key

# 4. Restart services
kubectl rollout restart statefulset/wasmbed-gateway -n wasmbed
kubectl rollout restart deployment/wasmbed-k8s-controller -n wasmbed

# 5. Verify recovery
./scripts/test.sh

echo " Disaster recovery completed!"
```

##  **Monitoring and Alerting**

### 1. **Enhanced Monitoring**

#### Prometheus Configuration
```yaml
apiVersion: monitoring.coreos.com/v1
kind: ServiceMonitor
metadata:
  name: wasmbed-monitoring
  namespace: monitoring
spec:
  selector:
    matchLabels:
      app: wasmbed-gateway
  endpoints:
  - port: metrics
    interval: 30s
    path: /metrics
  - port: health
    interval: 10s
    path: /health
---
apiVersion: monitoring.coreos.com/v1
kind: PrometheusRule
metadata:
  name: wasmbed-alerts
  namespace: monitoring
spec:
  groups:
  - name: wasmbed.rules
    rules:
    - alert: WasmbedGatewayDown
      expr: up{job="wasmbed-gateway"} == 0
      for: 1m
      labels:
        severity: critical
      annotations:
        summary: "Wasmbed Gateway is down"
        description: "Gateway {{ $labels.instance }} has been down for more than 1 minute"
    
    - alert: HighDeviceDisconnectionRate
      expr: rate(device_disconnections_total[5m]) > 0.1
      for: 2m
      labels:
        severity: warning
      annotations:
        summary: "High device disconnection rate"
        description: "Device disconnection rate is {{ $value }} per second"
```

### 2. **Grafana Dashboards**

#### Production Dashboard
```json
{
  "dashboard": {
    "title": "Wasmbed Production Dashboard",
    "panels": [
      {
        "title": "Gateway Health",
        "type": "stat",
        "targets": [
          {
            "expr": "up{job=\"wasmbed-gateway\"}",
            "legendFormat": "Gateway {{instance}}"
          }
        ]
      },
      {
        "title": "Connected Devices",
        "type": "gauge",
        "targets": [
          {
            "expr": "connected_devices_total",
            "legendFormat": "Connected Devices"
          }
        ]
      },
      {
        "title": "Application Deployments",
        "type": "timeseries",
        "targets": [
          {
            "expr": "rate(application_deployments_total[5m])",
            "legendFormat": "Deployments/sec"
          }
        ]
      }
    ]
  }
}
```

##  **Operational Procedures**

### 1. **Deployment Procedures**

#### Blue-Green Deployment
```bash
#!/bin/bash
# Blue-green deployment script
set -e

VERSION=$1
if [ -z "$VERSION" ]; then
    echo "Usage: $0 <version>"
    exit 1
fi

echo " Starting blue-green deployment for version $VERSION..."

# 1. Deploy new version (green)
kubectl set image deployment/wasmbed-gateway wasmbed-gateway=wasmbed-gateway:$VERSION -n wasmbed

# 2. Wait for rollout
kubectl rollout status deployment/wasmbed-gateway -n wasmbed --timeout=300s

# 3. Run health checks
./scripts/health-check.sh

# 4. Switch traffic (if health checks pass)
kubectl patch service wasmbed-gateway-service -n wasmbed -p '{"spec":{"selector":{"version":"'$VERSION'"}}}'

echo " Blue-green deployment completed!"
```

### 2. **Incident Response**

#### Incident Response Playbook
```yaml
# Incident response procedures
incident_types:
  - name: "Gateway Outage"
    severity: "Critical"
    response_time: "5 minutes"
    procedures:
      - "Check pod status: kubectl get pods -n wasmbed"
      - "Check logs: kubectl logs -n wasmbed deployment/wasmbed-gateway"
      - "Restart if needed: kubectl rollout restart deployment/wasmbed-gateway"
      - "Escalate if unresolved after 15 minutes"
  
  - name: "High Memory Usage"
    severity: "Warning"
    response_time: "30 minutes"
    procedures:
      - "Check metrics: kubectl top pods -n wasmbed"
      - "Scale up if needed: kubectl scale deployment/wasmbed-gateway --replicas=5"
      - "Investigate memory leaks in logs"
  
  - name: "Certificate Expiration"
    severity: "High"
    response_time: "1 hour"
    procedures:
      - "Check certificate validity: openssl x509 -in cert.crt -text -noout"
      - "Rotate certificates: ./scripts/rotate-certs.sh"
      - "Restart services after rotation"
```

##  **Security Testing**

### 1. **Penetration Testing**

#### Security Scan Script
```bash
#!/bin/bash
# Security testing script
set -e

echo " Starting security scan..."

# 1. Container vulnerability scan
trivy image wasmbed-gateway:latest --severity HIGH,CRITICAL

# 2. Kubernetes security scan
kube-bench --benchmark cis-1.6

# 3. Network policy validation
kubectl get networkpolicies -n wasmbed

# 4. RBAC audit
kubectl auth can-i --list -n wasmbed

# 5. Secret scanning
kubectl get secrets -n wasmbed -o yaml | grep -i "password\|key\|token"

echo " Security scan completed!"
```

### 2. **Compliance Checks**

#### SOC 2 Compliance Checklist
```yaml
compliance:
  soc2:
    cc1_control_environment:
      - "Access controls implemented"
      - "Change management procedures"
      - "Security awareness training"
    
    cc2_communication:
      - "Security policies documented"
      - "Incident response procedures"
      - "Change notification process"
    
    cc3_risk_assessment:
      - "Regular security assessments"
      - "Vulnerability scanning"
      - "Risk mitigation procedures"
    
    cc4_monitoring:
      - "Continuous monitoring"
      - "Log aggregation"
      - "Alert mechanisms"
    
    cc5_control_activities:
      - "Segregation of duties"
      - "System access controls"
      - "Data backup procedures"
```

##  **Implementation Checklist**

###  **Security Hardening**
- [ ] Production certificate management
- [ ] Network policies implementation
- [ ] RBAC hardening
- [ ] TLS configuration hardening
- [ ] Container security scanning

###  **High Availability**
- [ ] Multi-zone deployment
- [ ] Load balancer configuration
- [ ] Anti-affinity rules
- [ ] Leader election for controllers
- [ ] Health check endpoints

###  **Backup & Recovery**
- [ ] Automated backup procedures
- [ ] Disaster recovery plan
- [ ] Recovery testing
- [ ] Data retention policies
- [ ] Backup verification

###  **Monitoring & Alerting**
- [ ] Prometheus integration
- [ ] Grafana dashboards
- [ ] Alert rules configuration
- [ ] SLA monitoring
- [ ] Performance baselines

###  **Operational Procedures**
- [ ] Deployment procedures
- [ ] Incident response playbook
- [ ] Change management process
- [ ] Security testing procedures
- [ ] Compliance documentation

##  **Success Metrics**

### **Security Metrics**
- Zero critical vulnerabilities
- 100% certificate compliance
- <5 minute incident response time
- 99.9% security scan pass rate

### **Availability Metrics**
- 99.99% uptime SLA
- <30 second failover time
- Zero data loss during incidents
- 100% backup success rate

### **Performance Metrics**
- <100ms API response time
- <1 second deployment time
- 100% monitoring coverage
- <5 minute recovery time

---

**Phase 11 Status**: 🚧 **In Progress**  
**Production Readiness**: 🟡 **85% Complete**  
**Next Phase**:  **Phase 12: Ecosystem Development**
