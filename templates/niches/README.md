# Universal Niche System

The biomeOS niche system is **completely universal and agnostic**, designed to work with any combination of primals (current, future, or community-created) without requiring code modifications.

## Architecture

### Core Principles

1. **No Hardcoded Dependencies** - All niche definitions are loaded from YAML files
2. **Universal Primal Support** - Works with any primal type (toadstool, songbird, nestgate, beardog, squirrel, or future ones)
3. **Federation Aware** - Supports deployment across the self-contained federation system
4. **Community Extensible** - Anyone can create new niches by adding YAML files

### Primal Roles

| Primal | Role | Description |
|--------|------|-------------|
| **toadstool** | Compute | Container runtime, VM execution, substrate detection |
| **songbird** | Networking | Service discovery, orchestration, federation coordination |
| **nestgate** | Storage | ZFS storage, volume provisioning, data management |
| **beardog** | Security | Encryption, authentication, threat detection |
| **squirrel** | AI/MCP | AI agents, MCP protocol, intelligent automation |

## Creating a New Niche

### 1. Create YAML File

Create a new file in `niches/templates/your-niche.yaml`:

```yaml
niche:
  id: "your-niche-id"
  name: "Your Niche Name"
  description: "Description of what this niche provides"
  category: "custom"  # web_development, ai_research, gaming, custom, federation
  difficulty: "beginner"  # beginner, intermediate, advanced
  version: "1.0.0"
  author: "Your Name"
  
  features:
    - "Feature 1"
    - "Feature 2"
    - "Feature 3"
  
  required_primals:
    - "toadstool"  # Always include for compute
    # Add other required primals
  
  optional_primals:
    - "songbird"   # For networking
    - "nestgate"   # For storage
    - "squirrel"   # For AI/MCP
    - "beardog"    # For security
  
  customization_options:
    - id: "option_id"
      name: "Option Display Name"
      description: "What this option does"
      type: "select"  # select, text, number, boolean
      options:  # Only for select type
        - "Option 1"
        - "Option 2"
      default: "Option 1"
      required: true
      validation_regex: "^[a-zA-Z0-9-_]+$"  # Optional for text
      min: 1      # Optional for number
      max: 100    # Optional for number
  
  manifest_template: |
    metadata:
      version: "1.0.0"
      name: "{{project_name}}"
      description: "{{description}}"
      
    primals:
      compute:
        primal_type: "toadstool"
        version: ">=1.0.0"
        name: "{{project_name}}-compute"
        required: true
        config:
          container_runtime: "podman"
          # Your compute configuration
          
    services:
      main:
        name: "{{project_name}}-service"
        description: "Main service"
        primal: "compute"
        config:
          # Your service configuration
```

### 2. Template Variables

Use Handlebars syntax for dynamic content:

- `{{variable_name}}` - Simple substitution
- `{{#if variable}}...{{/if}}` - Conditional blocks
- `{{#if_eq variable "value"}}...{{/if_eq}}` - Equality conditions

### 3. Federation Support

For federation-aware niches, include:

```yaml
federation:
  deployment_strategy: "{{deployment_strategy}}"
  security_level: "{{security_level}}"
  max_latency_ms: {{max_latency_ms}}
  
primals:
  coordination:
    primal_type: "songbird"
    config:
      federation_enabled: true
      discovery_protocols: ["mdns", "upnp", "beardog"]
      
  security:
    primal_type: "beardog"
    config:
      security_level: "{{security_level}}"
      encryption_overhead_ms: 0.1  # Gaming-grade
```

## Example Niches

### Web Development
- **File**: `web-development.yaml`
- **Primals**: toadstool (compute) + songbird (networking)
- **Features**: Node.js, React/Vue/Angular, hot reload, databases

### AI Research
- **File**: `ai-research.yaml`
- **Primals**: toadstool (compute) + nestgate (storage)
- **Features**: GPU support, PyTorch/TensorFlow, Jupyter notebooks

### Federation Demo
- **File**: `federation-aware.yaml`
- **Primals**: songbird (coordination) + beardog (security) + toadstool (compute)
- **Features**: Multi-tower deployment, proximity-first placement, gaming-grade encryption

### Custom Generic
- **File**: `custom-generic.yaml`
- **Primals**: toadstool (compute) + optional others
- **Features**: Fully customizable, any primal combination

## System Architecture

### Loading Process

1. **Scan Directory** - System scans `niches/templates/` for `*.yaml` files
2. **Parse YAML** - Each file is parsed into a `NicheDefinition` structure
3. **Convert to Template** - YAML config is converted to internal `NicheTemplate` format
4. **Fallback** - If no YAML files found, uses built-in fallback templates

### Code Structure

```
ui/src/views/byob/
├── templates.rs        # Dynamic YAML loading system
├── types.rs           # Data structures
├── workflow.rs        # State management
├── rendering.rs       # UI rendering
└── mod.rs            # Module coordination
```

### Key Functions

- `get_available_niches()` - Loads all niche templates dynamically
- `load_niche_template()` - Parses individual YAML files
- `convert_yaml_to_template()` - Converts YAML to internal format

## Benefits

### Universal Support
- **No Code Changes** - New niches require only YAML files
- **Any Primal Combination** - Works with current and future primals
- **Community Extensible** - Easy for community to contribute niches

### Federation Ready
- **Multi-Tower Deployment** - Supports deployment across basement towers
- **Proximity-First** - Optimizes for local → regional → global placement
- **Security Integrated** - BearDog encryption and threat detection
- **Network Effects** - Benefits from federation scaling

### Developer Experience
- **Familiar YAML** - Standard configuration format
- **Template System** - Handlebars for dynamic content
- **Validation** - Built-in validation for options
- **Fallbacks** - Graceful degradation if files missing

## Future Extensions

### Community Marketplace
- Public registry of community-created niches
- Versioning and dependency management
- Quality ratings and reviews

### Advanced Features
- Multi-file niche definitions
- Nested template inheritance
- Dynamic resource calculation
- Real-time optimization

### Integration
- GitHub integration for niche sharing
- CI/CD pipeline templates
- Monitoring and analytics integration

## Contributing

To contribute a new niche:

1. Create your YAML file in `niches/templates/`
2. Test with the biomeOS UI
3. Submit a pull request with documentation
4. Community review and integration

The universal niche system makes biomeOS truly extensible and future-proof! 🌱 