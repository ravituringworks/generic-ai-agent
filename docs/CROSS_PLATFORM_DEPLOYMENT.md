# Cross-Platform Deployment Guide

Complete guide for deploying The Agency platform on Windows, Linux, and Kubernetes.

## Table of Contents

- [Windows Service](#windows-service)
- [Linux systemd](#linux-systemd)
- [Kubernetes](#kubernetes)
- [Docker](#docker)
- [Configuration](#configuration)
- [Troubleshooting](#troubleshooting)

---

## Windows Service

### Prerequisites

- Windows Server 2016+ or Windows 10+
- Administrator privileges
- Rust toolchain (for building)

### Building

```powershell
# Build the Windows service binary
cargo build --release --bin agency-service
```

### Installation

**Option 1: Using PowerShell Script (Recommended)**

```powershell
# Run as Administrator
.\scripts\install-windows.ps1
```

**Option 2: Manual Installation**

```powershell
# Create directories
New-Item -Path "C:\Program Files\TheAgency" -ItemType Directory -Force
New-Item -Path "C:\ProgramData\Agency" -ItemType Directory -Force

# Copy files
Copy-Item ".\target\release\agency-service.exe" -Destination "C:\Program Files\TheAgency\"
Copy-Item ".\config.example.toml" -Destination "C:\ProgramData\Agency\config.toml"

# Create service
sc.exe create AgencyService `
    binPath= "C:\Program Files\TheAgency\agency-service.exe" `
    DisplayName= "The Agency AI Platform" `
    start= auto
```

### Service Management

```powershell
# Start service
Start-Service AgencyService

# Stop service
Stop-Service AgencyService

# Check status
Get-Service AgencyService

# View logs
Get-EventLog -LogName Application -Source AgencyService -Newest 50
```

### Configuration

Configuration file location: `C:\ProgramData\Agency\config.toml`

```toml
[llm]
ollama_url = "http://localhost:11434"
model = "llama3.2:latest"

[memory]
database_url = "C:\\ProgramData\\Agency\\data\\memory.db"
```

### Uninstallation

```powershell
# Stop and remove service
Stop-Service AgencyService
sc.exe delete AgencyService

# Remove files
Remove-Item -Recurse -Force "C:\Program Files\TheAgency"
Remove-Item -Recurse -Force "C:\ProgramData\Agency"
```

---

## Linux systemd

### Prerequisites

- Linux distribution with systemd (Ubuntu 18.04+, CentOS 7+, Debian 10+, etc.)
- Root access or sudo privileges
- Rust toolchain (for building)

### Building

```bash
# Build the daemon binary
cargo build --release --bin agency-daemon
```

### Installation

**Option 1: Using Install Script (Recommended)**

```bash
# Run as root
sudo ./scripts/install-linux.sh
```

**Option 2: Manual Installation**

```bash
# Create system user
sudo useradd -r -s /bin/false -d /var/lib/the-agency agency

# Create directories
sudo mkdir -p /usr/local/bin
sudo mkdir -p /etc/the-agency
sudo mkdir -p /var/lib/the-agency
sudo mkdir -p /var/log/the-agency

# Copy files
sudo cp ./target/release/agency-daemon /usr/local/bin/
sudo cp ./config.example.toml /etc/the-agency/config.toml
sudo cp ./the-agency.service /etc/systemd/system/

# Set permissions
sudo chown -R agency:agency /var/lib/the-agency
sudo chown -R agency:agency /var/log/the-agency
sudo chmod 755 /usr/local/bin/agency-daemon

# Enable and start service
sudo systemctl daemon-reload
sudo systemctl enable the-agency
sudo systemctl start the-agency
```

### Service Management

```bash
# Start service
sudo systemctl start the-agency

# Stop service
sudo systemctl stop the-agency

# Restart service
sudo systemctl restart the-agency

# Check status
sudo systemctl status the-agency

# View logs
sudo journalctl -u the-agency -f

# View recent logs
sudo journalctl -u the-agency -n 100
```

### Configuration

Configuration file location: `/etc/the-agency/config.toml`

```toml
[llm]
ollama_url = "http://localhost:11434"
model = "llama3.2:latest"

[memory]
database_url = "/var/lib/the-agency/memory.db"
```

### Uninstallation

```bash
# Stop and disable service
sudo systemctl stop the-agency
sudo systemctl disable the-agency

# Remove files
sudo rm /etc/systemd/system/the-agency.service
sudo rm /usr/local/bin/agency-daemon
sudo rm -rf /etc/the-agency
sudo rm -rf /var/lib/the-agency
sudo rm -rf /var/log/the-agency

# Remove user
sudo userdel agency

# Reload systemd
sudo systemctl daemon-reload
```

---

## Kubernetes

### Prerequisites

- Kubernetes cluster (1.19+)
- kubectl configured
- Docker for building images
- Container registry access (optional)

### Building Docker Image

```bash
# Build image
docker build -t the-agency:v0.2.0 .

# Tag for registry (optional)
docker tag the-agency:v0.2.0 your-registry.com/the-agency:v0.2.0

# Push to registry (optional)
docker push your-registry.com/the-agency:v0.2.0
```

### Deployment

**Option 1: Using Deployment Script (Recommended)**

```bash
# Deploy to Kubernetes
./scripts/deploy-k8s.sh
```

**Option 2: Manual Deployment**

```bash
# Create namespace
kubectl apply -f k8s/namespace.yaml

# Create ConfigMap
kubectl apply -f k8s/configmap.yaml

# Create PVC
kubectl apply -f k8s/pvc.yaml

# Deploy application
kubectl apply -f k8s/deployment.yaml

# Create service
kubectl apply -f k8s/service.yaml

# Create HPA
kubectl apply -f k8s/hpa.yaml

# Create Ingress (optional)
kubectl apply -f k8s/ingress.yaml
```

### Kubernetes Management

```bash
# View pods
kubectl get pods -n the-agency

# View logs
kubectl logs -f deployment/the-agency -n the-agency

# Port forward to local machine
kubectl port-forward svc/the-agency 8080:8080 -n the-agency

# Scale deployment
kubectl scale deployment/the-agency --replicas=3 -n the-agency

# Update deployment
kubectl set image deployment/the-agency the-agency=the-agency:v0.2.1 -n the-agency

# Rollback deployment
kubectl rollout undo deployment/the-agency -n the-agency

# View rollout status
kubectl rollout status deployment/the-agency -n the-agency

# View deployment history
kubectl rollout history deployment/the-agency -n the-agency
```

### Configuration

Update the ConfigMap:

```bash
# Edit ConfigMap
kubectl edit configmap the-agency-config -n the-agency

# Or apply changes
kubectl apply -f k8s/configmap.yaml

# Restart pods to pick up changes
kubectl rollout restart deployment/the-agency -n the-agency
```

### Resource Management

```yaml
# Resource requests and limits in deployment.yaml
resources:
  requests:
    memory: "512Mi"
    cpu: "500m"
  limits:
    memory: "2Gi"
    cpu: "2000m"
```

### Scaling

**Manual Scaling:**

```bash
kubectl scale deployment/the-agency --replicas=5 -n the-agency
```

**Auto-scaling (HPA):**

The HorizontalPodAutoscaler automatically scales based on CPU/memory:

```yaml
# Configured in k8s/hpa.yaml
minReplicas: 2
maxReplicas: 10
targetCPUUtilizationPercentage: 70
targetMemoryUtilizationPercentage: 80
```

### Monitoring

```bash
# View HPA status
kubectl get hpa -n the-agency

# View resource usage
kubectl top pods -n the-agency
kubectl top nodes

# Describe pod for events
kubectl describe pod <pod-name> -n the-agency
```

### Uninstallation

```bash
# Delete entire namespace (removes everything)
kubectl delete namespace the-agency

# Or delete resources individually
kubectl delete -f k8s/
```

---

## Docker

### Building

```bash
# Build image
docker build -t the-agency:latest .

# Build with specific platform
docker buildx build --platform linux/amd64,linux/arm64 -t the-agency:latest .
```

### Running

```bash
# Run container
docker run -d \
  --name the-agency \
  -p 8080:8080 \
  -v /path/to/config:/etc/the-agency \
  -v /path/to/data:/var/lib/the-agency \
  the-agency:latest

# Run with custom config
docker run -d \
  --name the-agency \
  -p 8080:8080 \
  -v $(pwd)/config.toml:/etc/the-agency/config.toml \
  -v the-agency-data:/var/lib/the-agency \
  the-agency:latest

# View logs
docker logs -f the-agency

# Execute commands in container
docker exec -it the-agency /bin/sh
```

### Docker Compose

Create `docker-compose.yml`:

```yaml
version: '3.8'

services:
  the-agency:
    image: the-agency:v0.2.0
    build: .
    ports:
      - "8080:8080"
    volumes:
      - ./config.toml:/etc/the-agency/config.toml:ro
      - agency-data:/var/lib/the-agency
    environment:
      - RUST_LOG=info,the_agency=debug
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 3s
      retries: 3

volumes:
  agency-data:
```

Run with Docker Compose:

```bash
# Start services
docker-compose up -d

# View logs
docker-compose logs -f

# Stop services
docker-compose down

# Stop and remove volumes
docker-compose down -v
```

---

## Configuration

### Environment Variables

All platforms support these environment variables:

```bash
RUST_LOG=info,the_agency=debug  # Logging level
RUST_BACKTRACE=1                # Enable backtraces
```

### Configuration File

Common configuration options:

```toml
[agent]
name = "The Agency"
system_prompt = "You are a helpful AI assistant."
max_turns = 10

[llm]
model = "llama3.2:latest"
ollama_url = "http://localhost:11434"
temperature = 0.7
max_tokens = 2000

[memory]
database_url = "sqlite:memory.db"
embedding_model = "nomic-embed-text"
max_results = 5
similarity_threshold = 0.7

[mcp]
server_url = "http://localhost:3000"
timeout_seconds = 30
max_retries = 3

[a2a]
enabled = false
agent_id = "default-agent"
discovery_url = "http://localhost:8001"
listen_address = "0.0.0.0:8001"
```

---

## Troubleshooting

### Windows

**Service won't start:**
```powershell
# Check Event Viewer
Get-EventLog -LogName Application -Source AgencyService -Newest 20

# Verify binary path
sc.exe qc AgencyService

# Test binary manually
& "C:\Program Files\TheAgency\agency-service.exe"
```

**Port already in use:**
```powershell
# Find process using port 8080
netstat -ano | findstr :8080

# Kill process
taskkill /PID <pid> /F
```

### Linux

**Service fails to start:**
```bash
# Check logs
sudo journalctl -u the-agency -n 50 --no-pager

# Check service status
sudo systemctl status the-agency

# Verify binary
file /usr/local/bin/agency-daemon
```

**Permission issues:**
```bash
# Fix ownership
sudo chown -R agency:agency /var/lib/the-agency
sudo chown -R agency:agency /var/log/the-agency

# Check SELinux (if applicable)
sudo setenforce 0  # Temporarily disable
sudo ausearch -m avc -ts recent  # Check denials
```

### Kubernetes

**Pods not starting:**
```bash
# Describe pod
kubectl describe pod <pod-name> -n the-agency

# Check events
kubectl get events -n the-agency --sort-by='.lastTimestamp'

# View logs
kubectl logs <pod-name> -n the-agency
```

**Image pull errors:**
```bash
# Check image pull policy
kubectl get deployment the-agency -n the-agency -o yaml | grep imagePullPolicy

# Create image pull secret
kubectl create secret docker-registry regcred \
  --docker-server=<registry> \
  --docker-username=<username> \
  --docker-password=<password> \
  -n the-agency
```

**Persistent volume issues:**
```bash
# Check PVC status
kubectl get pvc -n the-agency

# Describe PVC
kubectl describe pvc the-agency-data -n the-agency

# Check storage class
kubectl get storageclass
```

### Common Issues

**Cannot connect to Ollama:**
- Verify Ollama is running
- Check `ollama_url` in config
- Test connection: `curl http://localhost:11434/api/tags`

**Database errors:**
- Ensure database directory exists and is writable
- Check disk space
- Verify database_url path

**API not responding:**
- Check if service is running
- Verify port is not blocked by firewall
- Test: `curl http://localhost:8080/health`

---

## Security Recommendations

### Windows
- Run service as limited service account, not LocalSystem
- Configure Windows Firewall rules
- Enable audit logging
- Use HTTPS with reverse proxy

### Linux
- Use dedicated `agency` user with minimal privileges
- Configure SELinux/AppArmor policies
- Enable firewalld/iptables rules
- Use systemd security features (see the-agency.service)

### Kubernetes
- Use Pod Security Standards
- Enable Network Policies
- Use Secrets for sensitive data
- Enable audit logging
- Use RBAC for access control
- Configure resource quotas

---

## Performance Tuning

### Resource Allocation

**Windows/Linux:**
- Adjust based on expected load
- Monitor CPU and memory usage
- Consider SSD for database

**Kubernetes:**
```yaml
resources:
  requests:    # Guaranteed resources
    memory: "512Mi"
    cpu: "500m"
  limits:      # Maximum resources
    memory: "2Gi"
    cpu: "2000m"
```

### Database Optimization

```toml
# Use persistent storage
[memory]
database_url = "sqlite:/path/to/persistent/memory.db"

# Adjust connection pool
max_connections = 10
connection_timeout = 30
```

### Horizontal Scaling (K8s)

- Use HPA for automatic scaling
- Deploy multiple replicas
- Consider StatefulSet for persistent state
- Use headless service for direct pod access

---

## Support

- 📚 [Main Documentation](../README.md)
- 🚀 [API Documentation](DEPLOYMENT.md)
- 🔄 [Saga Pattern Guide](API_IMPLEMENTATION.md)
- 🐛 [Report Issues](https://github.com/ravituringworks/the-agency/issues)

**Built with ❤️ in Rust**
