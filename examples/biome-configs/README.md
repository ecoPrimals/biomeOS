# biomeOS Sample Configurations

This directory contains sample `biome.yaml` configurations for different use cases, showcasing the power and flexibility of the biomeOS ecosystem.

## 📋 Available Configurations

### 1. 🤖 AI Research Biome (`ai-research-biome.yaml`)

**Use Case**: Machine learning research, GPU computing, and AI development

**Features**:
- **GPU Compute**: NVIDIA CUDA support with shared GPU memory
- **ML Frameworks**: TensorFlow, PyTorch, Jupyter Lab
- **Data Pipeline**: MLflow tracking, TensorBoard visualization
- **Distributed Training**: Horovod for multi-GPU training
- **AI Agents**: Research assistant with code generation capabilities
- **High-Performance Storage**: NVMe for models, SSD for datasets
- **Cost Optimization**: Auto-scaling and spot instances

**Key Services**:
- Jupyter Lab for interactive development
- MLflow for experiment tracking
- TensorBoard for visualization
- Distributed training coordinator
- AI research assistant agent
- Model serving API with Triton

**Resource Requirements**:
- CPU: 128 cores, 512GB RAM
- GPU: 8 NVIDIA GPUs with 32GB each
- Storage: 2TB NVMe + 10TB SSD + 100TB HDD
- Network: High-bandwidth for data transfer

### 2. 🔒 Secure Enterprise Biome (`secure-enterprise-biome.yaml`)

**Use Case**: Enterprise applications with maximum security, compliance, and audit requirements

**Features**:
- **Zero-Trust Architecture**: Continuous verification and least privilege
- **Multi-Factor Authentication**: Biometric, hardware tokens, certificates
- **Advanced Encryption**: AES-256-GCM, RSA-4096, HSM-backed keys
- **Compliance Frameworks**: SOC2, ISO27001, HIPAA, PCI-DSS, GDPR
- **Comprehensive Audit**: Immutable logs, 7-year retention
- **SIEM Integration**: Real-time threat detection and response
- **Data Governance**: Classification, lineage, retention policies

**Key Services**:
- Identity and Access Management (IAM)
- Security Information and Event Management (SIEM)
- Compliance management system
- Secure AI assistant with data residency
- Enterprise database with encryption
- Audit and governance dashboard

**Security Features**:
- Micro-segmentation and network policies
- Container security with gVisor
- Hardware Security Modules (HSMs)
- Behavioral analytics and anomaly detection
- Automated incident response

### 3. 🚀 Basic Development Biome (`basic-development-biome.yaml`)

**Use Case**: Simple development platform for getting started with biomeOS

**Features**:
- **Simple Setup**: Easy to understand and deploy
- **Web Development**: NGINX, Node.js, PostgreSQL
- **Development Tools**: Code-server, database admin
- **Basic AI**: Simple assistant for code help
- **Local Storage**: SSD-based storage with daily backups
- **Cost-Effective**: Auto-shutdown and resource optimization

**Key Services**:
- Web application (NGINX)
- Database (PostgreSQL)
- API backend (Node.js)
- Development tools (code-server)
- Simple AI assistant

**Resource Requirements**:
- CPU: 16 cores, 32GB RAM
- Storage: 500GB SSD
- Network: Standard bandwidth
- Budget: $100/month

## 🚀 Getting Started

### Prerequisites

1. **biomeOS Installation**: Ensure biomeOS is installed and configured
2. **Primal Dependencies**: Required primals (Toadstool, Songbird, NestGate, Squirrel, BearDog)
3. **Hardware Requirements**: Check resource requirements for your chosen configuration
4. **Network Access**: Ensure proper network connectivity

### Deployment Steps

1. **Choose Configuration**:
   ```bash
   # For AI research
   cp ai-research-biome.yaml my-biome.yaml
   
   # For enterprise security
   cp secure-enterprise-biome.yaml my-biome.yaml
   
   # For basic development
   cp basic-development-biome.yaml my-biome.yaml
   ```

2. **Customize Configuration**:
   - Update `metadata.team` with your team name
   - Modify resource limits based on your hardware
   - Adjust service configurations as needed
   - Configure authentication and security settings

3. **Deploy Biome**:
   ```bash
   # Validate configuration
   biomeos validate my-biome.yaml
   
   # Deploy biome
   biomeos deploy my-biome.yaml --team your-team
   
   # Check deployment status
   biomeos status --team your-team
   ```

4. **Access Services**:
   ```bash
   # List running services
   biomeos services --team your-team
   
   # Get service endpoints
   biomeos endpoints --team your-team
   ```

## 📊 Configuration Comparison

| Feature | AI Research | Enterprise | Basic Dev |
|---------|-------------|------------|-----------|
| **Security Level** | Standard | Maximum | Standard |
| **GPU Support** | ✅ Multi-GPU | ❌ CPU Only | ❌ CPU Only |
| **Storage Tiers** | 3 (NVMe/SSD/HDD) | 3 (Enterprise) | 1 (SSD) |
| **Multi-Instance** | ✅ On-demand | ✅ Persistent | ❌ Single |
| **Auto-Scaling** | ✅ Advanced | ✅ HA | ✅ Basic |
| **Monitoring** | ✅ ML Metrics | ✅ Security | ✅ Basic |
| **Backup** | ✅ Tiered | ✅ Immutable | ✅ Daily |
| **Cost** | $$$ High | $$$$ Premium | $ Low |
| **Complexity** | High | Very High | Low |

## 🔧 Customization Guide

### Common Customizations

1. **Resource Limits**:
   ```yaml
   # In primal configuration
   resource_limits:
     max_cpu_cores: 16    # Adjust based on hardware
     max_memory: "32GB"   # Adjust based on RAM
     max_storage: "1TB"   # Adjust based on storage
   ```

2. **Security Level**:
   ```yaml
   # In security configuration
   security:
     level: "standard"     # or "high" or "maximum"
   ```

3. **Service Scaling**:
   ```yaml
   # In service configuration
   scaling:
     min_replicas: 1
     max_replicas: 5
     auto_scaling:
       enabled: true
       target_cpu: "80%"
   ```

4. **Storage Configuration**:
   ```yaml
   # In storage configuration
   storage:
     my-volume:
       tier: "ssd"         # or "nvme" or "hdd"
       size: "100GB"
       backup: true
   ```

### Advanced Customizations

1. **Custom AI Providers**:
   ```yaml
   # In Squirrel configuration
   ai_providers:
     - name: "custom-llm"
       type: "custom"
       endpoint: "https://my-llm-api.com"
       models: ["my-model"]
   ```

2. **Custom Security Policies**:
   ```yaml
   # In BearDog configuration
   authorization:
     policies:
       - name: "custom-role"
         subjects: ["group:my-team"]
         resources: ["my-service"]
         actions: ["read", "write"]
   ```

3. **Custom Monitoring**:
   ```yaml
   # In monitoring configuration
   monitoring:
     custom_metrics:
       - name: "my-metric"
         query: "my_custom_query"
         alert_threshold: 90
   ```

## 🔍 Configuration Validation

### Validation Commands

```bash
# Validate syntax
biomeos validate my-biome.yaml

# Check resource requirements
biomeos check-resources my-biome.yaml

# Dry-run deployment
biomeos deploy my-biome.yaml --dry-run

# Validate security settings
biomeos security-scan my-biome.yaml
```

### Common Validation Errors

1. **Resource Limits Exceeded**:
   ```
   Error: CPU limit (32 cores) exceeds available (16 cores)
   Solution: Reduce max_cpu_cores in primal configuration
   ```

2. **Invalid Storage Tier**:
   ```
   Error: Storage tier 'nvme' not available
   Solution: Use available tier or configure storage provider
   ```

3. **Security Policy Conflict**:
   ```
   Error: Security policy conflicts with compliance requirements
   Solution: Adjust security level or policy configuration
   ```

## 🏗️ Creating Custom Configurations

### 1. Start with a Template

Choose the closest template to your use case:
- AI Research: For compute-intensive, GPU-based workloads
- Enterprise: For security-critical, compliance-focused deployments
- Basic Development: For simple, cost-effective development environments

### 2. Customize Primals

Each primal can be customized for your specific needs:

```yaml
primals:
  toadstool:
    config:
      # Custom runtime configuration
      runtime_preferences: ["container", "wasm"]
      
  songbird:
    config:
      # Custom service mesh configuration
      service_mesh:
        protocol: "grpc"
        
  nestgate:
    config:
      # Custom storage configuration
      storage_tiers:
        - name: "custom-tier"
          type: "nvme"
          capacity: "1TB"
```

### 3. Define Services

Add your custom services:

```yaml
services:
  my-service:
    primal: "toadstool"
    runtime: "container"
    
    container:
      image: "my-registry/my-app:latest"
      ports:
        - "8080:8080"
        
    resources:
      cpu: "2"
      memory: "4GB"
      
    health_check:
      http:
        path: "/health"
        port: 8080
```

### 4. Configure Resources

Define your resource requirements:

```yaml
resources:
  storage:
    my-data:
      tier: "ssd"
      size: "500GB"
      backup: true
      
  networking:
    service_mesh:
      enabled: true
      
  compute:
    cpu_pool:
      cores: 32
      memory: "128GB"
```

## 📚 Best Practices

### 1. Security
- Always use the appropriate security level for your use case
- Enable encryption at rest and in transit
- Configure proper authentication and authorization
- Regular security updates and vulnerability scanning

### 2. Resource Management
- Set appropriate resource limits to prevent resource exhaustion
- Use auto-scaling to handle variable workloads
- Monitor resource usage and optimize accordingly
- Implement proper backup and disaster recovery

### 3. Monitoring
- Enable comprehensive monitoring and alerting
- Set up proper log aggregation and analysis
- Configure health checks for all services
- Implement performance monitoring and optimization

### 4. Cost Optimization
- Use auto-shutdown for development environments
- Implement resource optimization policies
- Monitor costs and set budget alerts
- Use spot instances where appropriate

## 🐛 Troubleshooting

### Common Issues

1. **Deployment Failures**:
   - Check resource availability
   - Verify primal connectivity
   - Review configuration syntax
   - Check service dependencies

2. **Service Connectivity Issues**:
   - Verify network policies
   - Check service mesh configuration
   - Validate DNS settings
   - Review firewall rules

3. **Performance Issues**:
   - Monitor resource usage
   - Check storage performance
   - Optimize network configuration
   - Review scaling policies

### Debug Commands

```bash
# Check deployment status
biomeos status --team your-team --verbose

# View service logs
biomeos logs service-name --team your-team

# Check resource usage
biomeos resources --team your-team

# Troubleshoot connectivity
biomeos network-test --team your-team
```

## 🆘 Support

For additional support:

1. **Documentation**: Check the main biomeOS documentation
2. **Community**: Join the biomeOS community forums
3. **Issues**: Report issues on the GitHub repository
4. **Professional Support**: Contact the biomeOS team for enterprise support

## 🔄 Updates

These configurations are regularly updated to reflect:
- New biomeOS features
- Security improvements
- Performance optimizations
- Best practice updates

Check the git history for recent changes and update your configurations accordingly.

---

**Note**: These sample configurations are provided as starting points. Always customize them according to your specific requirements, security policies, and resource constraints. 