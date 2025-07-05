# biomeOS Source Management System Specification

## Overview

The biomeOS Source Management System (SMS) is a comprehensive solution for managing, distributing, and integrating software components across the biomeOS ecosystem. It handles everything from Primal components to custom user applications, supporting multiple source types and distribution mechanisms.

## Core Philosophy

**"Universal Source Sovereignty"** - biomeOS should be able to pull, build, and integrate software from any source while maintaining sovereignty and security. Whether it's a GitHub repository, a local development folder, an enterprise GitLab instance, or a custom distribution mechanism, biomeOS adapts to existing workflows rather than forcing users to adapt to it.

## System Architecture

### Layer 0: Source Discovery & Authentication
- **Multi-Protocol Support**: Git (HTTPS/SSH), Local filesystem, HTTP/HTTPS APIs, Custom protocols
- **Authentication Management**: SSH keys, OAuth tokens, API keys, certificate-based auth
- **Repository Metadata**: Automatic discovery of biome.yaml, Primal specifications, build configurations

### Layer 1: Component Classification & Validation
- **Primal Detection**: Automatic identification of BearDog, Songbird, NestGate, Toadstool, Squirrel components
- **Biome Validation**: Parse and validate biome.yaml manifests
- **Dependency Analysis**: Build dependency graphs, detect conflicts, version compatibility
- **Security Scanning**: MYCORRHIZA integration for sovereignty compliance

### Layer 2: Build & Package Management
- **Universal Build System**: Support for Cargo, Docker, npm, custom build scripts
- **Artifact Management**: Binaries, containers, WASM modules, configuration files
- **Version Management**: Semantic versioning, branch/tag tracking, release channels
- **Distribution Packaging**: ISO generation, package repositories, update mechanisms

### Layer 3: Integration & Deployment
- **Runtime Integration**: Automatic Toadstool registration, service discovery
- **Configuration Management**: Environment-specific configurations, secrets management
- **Health Monitoring**: Component health checks, dependency validation
- **Rollback Mechanisms**: Safe deployment with automatic rollback capabilities

## Source Types

### 1. Git Repositories
```yaml
source:
  type: git
  url: "https://github.com/owner/repo.git"
  branch: "main"
  tag: "v1.0.0"
  commit: "abc123"
  auth:
    type: ssh_key
    key_path: "~/.ssh/biomeos_key"
```

### 2. Local Development
```yaml
source:
  type: local
  path: "/home/user/my-primal"
  watch: true  # Auto-reload on changes
  build_command: "cargo build --release"
```

### 3. HTTP/HTTPS Archives
```yaml
source:
  type: http
  url: "https://releases.example.com/primal-v1.0.0.tar.gz"
  checksum: "sha256:abc123..."
  auth:
    type: bearer_token
    token: "${RELEASE_TOKEN}"
```

### 4. Container Registries
```yaml
source:
  type: container
  registry: "ghcr.io"
  image: "owner/biome-component:latest"
  auth:
    type: token
    token: "${GITHUB_TOKEN}"
```

### 5. Custom Protocols
```yaml
source:
  type: custom
  protocol: "enterprise-vault"
  endpoint: "vault.company.com"
  path: "/biome-components/secure-primal"
  auth:
    type: certificate
    cert_path: "/etc/biomeos/certs/enterprise.pem"
```

## Component Management

### Primal Components
- **Auto-Discovery**: Scan repositories for Primal indicators (Cargo.toml with specific features, directory structure)
- **Compatibility Matrix**: Track which Primal versions work with which biomeOS versions
- **Integration Templates**: Automatic generation of integration code for new Primals
- **Performance Profiling**: Benchmark Primal components for resource usage

### Biome Applications
- **Manifest Validation**: Comprehensive biome.yaml parsing and validation
- **Dependency Resolution**: Automatic resolution of Primal dependencies
- **Resource Allocation**: Predict and manage resource requirements
- **Specialization Support**: Templates for different biome types (AI, enterprise, edge)

### Custom Extensions
- **Plugin Architecture**: Support for custom source plugins
- **Transformation Pipelines**: Custom build and packaging workflows
- **Compatibility Shims**: Automatic generation of compatibility layers
- **Documentation Generation**: Auto-generate integration docs

## Distribution Mechanisms

### 1. ISO Distribution
```yaml
distribution:
  type: iso
  base_image: "biomeos-core-v1.0.0.iso"
  components:
    - name: "beardog"
      version: "latest"
      source: "github:biomeos/beardog"
    - name: "custom-primal"
      version: "v2.1.0"
      source: "local:/path/to/primal"
  output: "biomeos-custom-v1.0.0.iso"
```

### 2. Package Repositories
```yaml
distribution:
  type: repository
  url: "https://packages.biomeos.org"
  channels:
    - stable
    - beta
    - nightly
  signing_key: "/etc/biomeos/keys/repo-signing.key"
```

### 3. Container Images
```yaml
distribution:
  type: container
  base_image: "biomeos/runtime:latest"
  registry: "ghcr.io/biomeos"
  tags:
    - "latest"
    - "v1.0.0"
    - "stable"
```

## Security & Sovereignty

### Source Verification
- **Cryptographic Signatures**: Verify all source code with digital signatures
- **Reproducible Builds**: Ensure identical builds from identical sources
- **Supply Chain Security**: Track all dependencies and their sources
- **Vulnerability Scanning**: Automatic CVE scanning for all components

### MYCORRHIZA Integration
- **Source Sovereignty**: Classify sources as trusted/untrusted
- **Energy Flow Management**: Control which sources can access which resources
- **Audit Trail**: Complete logging of all source operations
- **Compliance Reporting**: Generate compliance reports for enterprise use

### Private Repository Support
- **Enterprise Integration**: Support for private GitLab, GitHub Enterprise, Bitbucket
- **VPN/Firewall Traversal**: Work behind corporate firewalls
- **Air-Gapped Environments**: Support for completely offline environments
- **Custom Certificate Authorities**: Support for internal CAs

## Configuration Management

### Global Configuration
```yaml
# ~/.config/biomeos/sources.yaml
sources:
  github:
    auth:
      type: token
      token: "${GITHUB_TOKEN}"
    cache_dir: "/var/cache/biomeos/github"
  
  gitlab:
    url: "https://gitlab.company.com"
    auth:
      type: ssh_key
      key_path: "~/.ssh/company_key"
  
  local:
    watch_directories:
      - "/home/user/biomeos-dev"
      - "/opt/biomeos/local-primals"

build:
  parallel_jobs: 4
  cache_size: "10GB"
  timeout: "30m"

security:
  require_signatures: true
  allowed_sources:
    - "github.com/biomeos/*"
    - "gitlab.company.com/biomeos/*"
    - "local:/opt/biomeos/*"
```

### Component Configuration
```yaml
# biome.yaml - Enhanced source management
metadata:
  name: "ai-research-biome"
  version: "1.0.0"

sources:
  primals:
    beardog:
      source: "github:biomeos/beardog"
      version: "v2.0.0"
      build_args:
        - "--features=enterprise"
    
    custom_ml_primal:
      source: "gitlab:company/ml-primal"
      version: "main"
      build_command: "make release"
      
  applications:
    jupyter:
      source: "local:/home/user/jupyter-biomeos"
      watch: true
    
    data_processor:
      source: "https://releases.company.com/data-processor-v1.0.0.tar.gz"
      checksum: "sha256:def456..."

build:
  steps:
    - name: "Build Primals"
      command: "biomeos-build primals"
    - name: "Configure Services"
      command: "biomeos-configure services"
    - name: "Run Tests"
      command: "biomeos-test integration"
```

## CLI Interface

### Source Management Commands
```bash
# Add a new source
biomeos source add github https://github.com/owner/repo.git

# List all sources
biomeos source list

# Update source metadata
biomeos source update <source-name>

# Remove a source
biomeos source remove <source-name>

# Search for components
biomeos source search "neural network"
```

### Build Commands
```bash
# Build a specific component
biomeos build component <component-name>

# Build entire biome
biomeos build biome /path/to/biome.yaml

# Build for specific target
biomeos build --target x86_64-unknown-linux-gnu

# Build with custom configuration
biomeos build --config production.yaml
```

### Distribution Commands
```bash
# Create ISO
biomeos dist iso --config biome.yaml --output biomeos-custom.iso

# Publish to repository
biomeos dist publish --channel stable --version v1.0.0

# Generate container image
biomeos dist container --base-image biomeos/runtime:latest
```

## API Interface

### RESTful API
```yaml
# Source Management
GET    /api/v1/sources
POST   /api/v1/sources
PUT    /api/v1/sources/{id}
DELETE /api/v1/sources/{id}

# Component Management
GET    /api/v1/components
GET    /api/v1/components/{id}
POST   /api/v1/components/{id}/build
GET    /api/v1/components/{id}/status

# Build Management
GET    /api/v1/builds
POST   /api/v1/builds
GET    /api/v1/builds/{id}
GET    /api/v1/builds/{id}/logs

# Distribution
POST   /api/v1/distributions/iso
POST   /api/v1/distributions/container
POST   /api/v1/distributions/repository
```

### GraphQL API
```graphql
type Source {
  id: ID!
  name: String!
  type: SourceType!
  url: String!
  auth: AuthConfig
  components: [Component!]!
}

type Component {
  id: ID!
  name: String!
  type: ComponentType!
  version: String!
  source: Source!
  dependencies: [Component!]!
  builds: [Build!]!
}

type Build {
  id: ID!
  component: Component!
  status: BuildStatus!
  logs: String
  artifacts: [Artifact!]!
  createdAt: DateTime!
}
```

## Implementation Phases

### Phase 1: Core Source Management (Weeks 1-2)
- Basic Git repository support
- Local filesystem sources
- Simple component discovery
- Basic CLI interface

### Phase 2: Build System Integration (Weeks 3-4)
- Cargo/Rust build integration
- Docker container support
- Artifact management
- Build caching

### Phase 3: Advanced Sources (Weeks 5-6)
- HTTP/HTTPS archive support
- Container registry integration
- Custom protocol support
- Authentication management

### Phase 4: Distribution & Packaging (Weeks 7-8)
- ISO generation
- Package repository support
- Container image creation
- Update mechanisms

### Phase 5: Enterprise Features (Weeks 9-10)
- Private repository support
- Advanced security features
- Compliance reporting
- Air-gapped operation

### Phase 6: Advanced Integration (Weeks 11-12)
- MYCORRHIZA integration
- Advanced dependency management
- Performance optimization
- Monitoring & alerting

## Success Metrics

### Technical Metrics
- **Source Discovery Time**: < 5 seconds for any repository
- **Build Performance**: 80% cache hit rate, < 10 minutes for full builds
- **Distribution Size**: < 20% overhead for packaged distributions
- **Security Coverage**: 100% vulnerability scanning, 0 false positives

### User Experience Metrics
- **Setup Time**: < 5 minutes from clone to first build
- **Learning Curve**: < 1 hour to understand basic concepts
- **Documentation Coverage**: 100% API coverage, 90% tutorial coverage
- **Error Recovery**: < 30 seconds to recover from common errors

### Ecosystem Metrics
- **Component Compatibility**: 95% compatibility across versions
- **Source Diversity**: Support for 10+ different source types
- **Enterprise Adoption**: 50+ enterprise deployments
- **Community Contributions**: 100+ community-submitted components

## Future Enhancements

### Advanced Features
- **AI-Powered Component Discovery**: Automatically discover and classify components
- **Blockchain-Based Verification**: Immutable source verification
- **Federated Source Networks**: Distributed source management
- **Predictive Dependency Management**: Predict and prevent dependency conflicts

### Integration Opportunities
- **CI/CD Integration**: GitHub Actions, GitLab CI, Jenkins plugins
- **IDE Extensions**: VS Code, IntelliJ, Vim plugins
- **Cloud Provider Integration**: AWS, GCP, Azure native support
- **Kubernetes Operators**: Native Kubernetes component management

This specification provides a comprehensive foundation for building a robust, scalable, and secure source management system that aligns with biomeOS's sovereignty-first philosophy while providing enterprise-grade functionality. 