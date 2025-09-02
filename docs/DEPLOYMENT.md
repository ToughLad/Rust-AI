# Deployment Guide

## Overview

This guide covers deploying Rust-AI in various environments, from development to production-scale deployments.

---

## Quick Deploy Options

### Option 1: Docker (Recommended)

**Build and Run:**
```bash
# Build image
docker build -t rust-ai .

# Run container
docker run -p 3000:3000 --env-file .env rust-ai
```

**Docker Compose:**
```yaml
version: '3.8'
services:
  rust-ai:
    build: .
    ports:
      - "3000:3000"
    environment:
      - BIND_ADDRESS=0.0.0.0:3000
      - ACTION_TOKEN_SECRET=${ACTION_TOKEN_SECRET}
      - OPENAI_API_KEY=${OPENAI_API_KEY}
    restart: unless-stopped
```

### Option 2: Direct Binary

```bash
# Build release binary
cargo build --release

# Run with environment
./target/release/rust-ai
```

---

## Production Deployment

### Prerequisites

#### System Requirements
- **CPU**: 2+ cores recommended
- **RAM**: 1GB minimum, 2GB+ recommended  
- **Storage**: 10GB minimum for logs and caching
- **Network**: HTTPS-capable reverse proxy

#### Dependencies
- **Rust 1.70+** (for building)
- **SSL/TLS certificates** (for HTTPS)
- **Database**: Convex or compatible backend
- **Cache**: Redis (optional but recommended)

### Environment Setup

#### Production Environment Variables
```bash
# Server Configuration
BIND_ADDRESS=127.0.0.1:3000
RUST_ENV=production
RUST_LOG=info

# Security
ACTION_TOKEN_SECRET=<64-character-random-string>

# Database
CONVEX_URL=https://your-production.convex.cloud

# Provider APIs (add as needed)
OPENAI_API_KEY=sk-...
ANTHROPIC_API_KEY=sk-ant-...
# ... other provider keys
```

#### Generating Secure Secrets
```bash
# Generate JWT secret
openssl rand -base64 64

# Generate API keys (if needed)
uuidgen | tr -d '-' | tr '[:upper:]' '[:lower:]'
```

---

## Cloud Deployment

### AWS Deployment

#### Using ECS (Elastic Container Service)

**Dockerfile:**
```dockerfile
FROM rust:1.75-slim as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && \
    apt-get install -y ca-certificates && \
    rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/rust-ai /usr/local/bin/
EXPOSE 3000
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:3000/health || exit 1
CMD ["rust-ai"]
```

**ECS Task Definition:**
```json
{
  "family": "rust-ai",
  "networkMode": "awsvpc",
  "requiresCompatibilities": ["FARGATE"],
  "cpu": "512",
  "memory": "1024",
  "executionRoleArn": "arn:aws:iam::account:role/ecsTaskExecutionRole",
  "containerDefinitions": [
    {
      "name": "rust-ai",
      "image": "your-account.dkr.ecr.region.amazonaws.com/rust-ai:latest",
      "portMappings": [
        {
          "containerPort": 3000,
          "protocol": "tcp"
        }
      ],
      "environment": [
        {"name": "BIND_ADDRESS", "value": "0.0.0.0:3000"},
        {"name": "RUST_ENV", "value": "production"}
      ],
      "secrets": [
        {
          "name": "ACTION_TOKEN_SECRET",
          "valueFrom": "arn:aws:secretsmanager:region:account:secret:rust-ai/jwt-secret"
        }
      ],
      "logConfiguration": {
        "logDriver": "awslogs",
        "options": {
          "awslogs-group": "/ecs/rust-ai",
          "awslogs-region": "us-east-1",
          "awslogs-stream-prefix": "ecs"
        }
      }
    }
  ]
}
```

### Google Cloud Platform

#### Using Cloud Run

```bash
# Build and push container
docker build -t gcr.io/PROJECT-ID/rust-ai .
docker push gcr.io/PROJECT-ID/rust-ai

# Deploy to Cloud Run
gcloud run deploy rust-ai \
  --image gcr.io/PROJECT-ID/rust-ai \
  --platform managed \
  --region us-central1 \
  --allow-unauthenticated \
  --port 3000 \
  --memory 1Gi \
  --cpu 1 \
  --set-env-vars RUST_ENV=production \
  --set-secrets ACTION_TOKEN_SECRET=rust-ai-jwt:latest
```

### Digital Ocean

#### Using App Platform

**app.yaml:**
```yaml
name: rust-ai
services:
- name: api
  source_dir: /
  github:
    repo: your-username/rust-ai
    branch: main
  run_command: ./target/release/rust-ai
  environment_slug: rust
  instance_count: 1
  instance_size_slug: basic-xxs
  http_port: 3000
  envs:
  - key: BIND_ADDRESS
    value: 0.0.0.0:3000
  - key: RUST_ENV
    value: production
  - key: ACTION_TOKEN_SECRET
    value: your-jwt-secret
    type: SECRET
```

---

## Reverse Proxy Setup

### Nginx Configuration

```nginx
server {
    listen 80;
    server_name yourdomain.com;
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl http2;
    server_name yourdomain.com;

    ssl_certificate /path/to/cert.pem;
    ssl_certificate_key /path/to/private.key;
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers HIGH:!aNULL:!MD5;

    # Rate limiting
    limit_req_zone $binary_remote_addr zone=api:10m rate=10r/s;
    limit_req zone=api burst=20 nodelay;

    location / {
        proxy_pass http://127.0.0.1:3000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_cache_bypass $http_upgrade;
        
        # Timeouts
        proxy_connect_timeout 60s;
        proxy_send_timeout 60s;
        proxy_read_timeout 60s;
    }

    # Health check endpoint (no rate limiting)
    location /health {
        proxy_pass http://127.0.0.1:3000/health;
        access_log off;
    }
}
```

### Traefik Configuration

**docker-compose.yml:**
```yaml
version: '3.8'
services:
  traefik:
    image: traefik:v2.10
    command:
      - --api.dashboard=true
      - --entrypoints.websecure.address=:443
      - --providers.docker=true
      - --certificatesresolvers.letsencrypt.acme.email=you@example.com
      - --certificatesresolvers.letsencrypt.acme.storage=/letsencrypt/acme.json
      - --certificatesresolvers.letsencrypt.acme.tlschallenge=true
    ports:
      - "443:443"
      - "8080:8080"
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock
      - ./letsencrypt:/letsencrypt

  rust-ai:
    build: .
    labels:
      - traefik.enable=true
      - traefik.http.routers.rust-ai.rule=Host(`yourdomain.com`)
      - traefik.http.routers.rust-ai.tls.certresolver=letsencrypt
      - traefik.http.services.rust-ai.loadbalancer.server.port=3000
    environment:
      - BIND_ADDRESS=0.0.0.0:3000
```

---

## Monitoring & Observability

### Health Checks

The `/health` endpoint provides service status:

```bash
# Basic health check
curl https://yourdomain.com/health

# Expected response
{
  "status": "healthy",
  "timestamp": "2025-01-02T12:00:00Z"
}
```

### Logging Setup

#### Structured Logging
```bash
# Enable JSON logging for production
RUST_LOG=info,rust_ai=debug cargo run
```

#### Log Aggregation (ELK Stack)
```yaml
version: '3.8'
services:
  elasticsearch:
    image: docker.elastic.co/elasticsearch/elasticsearch:8.8.0
    environment:
      - discovery.type=single-node
      - xpack.security.enabled=false
    
  kibana:
    image: docker.elastic.co/kibana/kibana:8.8.0
    environment:
      - ELASTICSEARCH_URL=http://elasticsearch:9200

  rust-ai:
    build: .
    logging:
      driver: "json-file"
      options:
        max-size: "10m"
        max-file: "3"
```

### Metrics Collection

#### Prometheus Metrics (Future Enhancement)
```rust
// Add to Cargo.toml dependencies
prometheus = "0.13"
axum-prometheus = "0.4"

// In main.rs
use axum_prometheus::PrometheusMetricLayer;

let (prometheus_layer, metric_handle) = PrometheusMetricLayer::pair();
let app = Router::new()
    .route("/metrics", get(|| async move { metric_handle.render() }))
    .layer(prometheus_layer);
```

---

## Security Hardening

### Application Security

#### Environment Variables
```bash
# Never log sensitive data
RUST_LOG=info  # Not debug/trace in production

# Use secrets management
# AWS: Systems Manager Parameter Store
# GCP: Secret Manager  
# Azure: Key Vault
```

#### Rate Limiting
```rust
// In application or reverse proxy
// Anonymous: 5 requests/day
// Registered: Based on tier
// Per IP: 100 requests/minute
```

### Infrastructure Security

#### Firewall Rules
```bash
# Only allow necessary ports
ufw allow 22   # SSH
ufw allow 80   # HTTP (redirect)
ufw allow 443  # HTTPS
ufw deny 3000  # Direct app access
ufw enable
```

#### SSL/TLS Configuration
```bash
# Use modern TLS versions
ssl_protocols TLSv1.2 TLSv1.3;

# Strong cipher suites
ssl_ciphers ECDHE+AESGCM:ECDHE+CHACHA20:DHE+AESGCM:DHE+CHACHA20:!aNULL:!MD5:!DSS;

# Enable HSTS
add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;
```

---

## Performance Optimization

### Application Tuning

#### Rust Compiler Optimizations
```toml
# Cargo.toml
[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
panic = "abort"
```

#### Runtime Configuration
```bash
# Increase file descriptor limits
ulimit -n 65536

# Configure memory limits
RUST_MAX_STACK=8388608
```

### Scaling Strategies

#### Horizontal Scaling
```yaml
# Kubernetes deployment
apiVersion: apps/v1
kind: Deployment
metadata:
  name: rust-ai
spec:
  replicas: 3
  selector:
    matchLabels:
      app: rust-ai
  template:
    metadata:
      labels:
        app: rust-ai
    spec:
      containers:
      - name: rust-ai
        image: rust-ai:latest
        ports:
        - containerPort: 3000
        resources:
          requests:
            memory: "512Mi"
            cpu: "250m"
          limits:
            memory: "1Gi"
            cpu: "500m"
```

#### Load Balancing
```nginx
upstream rust_ai_backend {
    server 127.0.0.1:3001;
    server 127.0.0.1:3002;
    server 127.0.0.1:3003;
    
    # Health checks
    check interval=3000 rise=2 fall=5 timeout=1000;
}

server {
    location / {
        proxy_pass http://rust_ai_backend;
    }
}
```

---

## Maintenance

### Updates and Rollbacks

#### Rolling Updates
```bash
# Build new version
docker build -t rust-ai:v1.1.0 .

# Deploy with zero downtime
docker-compose up -d --no-deps rust-ai

# Verify deployment
curl https://yourdomain.com/health
```

#### Database Migrations
```bash
# Plan migrations carefully
# Test in staging environment first
# Have rollback plan ready
```

### Backup Strategies

#### Database Backup
```bash
# Convex provides automatic backups
# Verify backup/restore procedures regularly
```

#### Configuration Backup
```bash
# Backup environment configuration
# Store in version control (without secrets)
# Use infrastructure as code (Terraform, etc.)
```

---

## Troubleshooting

### Common Issues

#### High CPU Usage
```bash
# Check for infinite loops
# Monitor provider API response times
# Verify rate limiting is working

# Use profiling tools
perf record -g ./target/release/rust-ai
perf report
```

#### Memory Leaks
```bash
# Monitor memory usage
watch -n 1 'cat /proc/$(pgrep rust-ai)/status | grep VmRSS'

# Use memory profilers
valgrind --tool=massif ./target/release/rust-ai
```

#### Network Issues
```bash
# Test provider connectivity
curl -v https://api.openai.com/v1/models

# Check DNS resolution
nslookup api.anthropic.com

# Monitor network latency
ping -c 5 api.mistral.ai
```

### Logs Analysis

#### Error Patterns
```bash
# Find authentication errors
grep "401\|authentication" /var/log/rust-ai.log

# Find provider errors  
grep "provider.*error" /var/log/rust-ai.log

# Find slow requests
grep "response_time.*[5-9][0-9][0-9][0-9]" /var/log/rust-ai.log
```

#### Performance Analysis
```bash
# Analyze response times
awk '/response_time/ {sum+=$3; count++} END {print "Average:", sum/count}' /var/log/rust-ai.log

# Find most used providers
grep "provider:" /var/log/rust-ai.log | awk '{print $2}' | sort | uniq -c | sort -nr
```