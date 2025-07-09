# biomeOS Universal UI

**AI-first, API-driven user interface for the entire biomeOS ecosystem**

The biomeOS Universal UI is a modern, intelligent interface that works seamlessly across all Primals in the biomeOS ecosystem. Whether you're using standard Primals, custom implementations, or community forks, this UI provides a consistent, powerful experience.

## 🌟 Key Features

### 🤖 AI-First Design
- **Natural Language Commands**: Interact with your entire ecosystem using conversational AI
- **Context-Aware Responses**: AI understands your ecosystem state and provides relevant suggestions
- **Intelligent Routing**: Commands automatically route to the appropriate Primals
- **Helpful Suggestions**: Get proactive recommendations for next steps

### 🔧 Universal Primal Support
- **Works with Any Primal**: Standard, custom, forked, or community-developed Primals
- **Automatic Discovery**: Detects and connects to available Primals automatically
- **Capability Detection**: Automatically discovers what each Primal can do
- **Graceful Degradation**: Continues working even when some Primals are unavailable

### 🎨 Multi-Mode Interface
- **Desktop App**: Native desktop experience using Tauri
- **Terminal UI**: Rich terminal interface using ratatui
- **Web Interface**: Browser-based UI using modern web technologies
- **CLI Mode**: Command-line interface for automation and scripting

### 📊 Real-Time Features
- **Live Monitoring**: Real-time ecosystem status and health monitoring
- **Event Streaming**: Live events from all Primals in a unified stream
- **Automatic Updates**: UI updates automatically as your ecosystem changes
- **Performance Metrics**: Monitor resource usage and performance across Primals

## 🚀 Quick Start

### Installation

```bash
# Clone the repository
git clone https://github.com/sovereignscience/biomeOS
cd biomeOS

# Build the UI
cargo build --package biomeos-ui --release

# Run the UI
./target/release/biomeos-ui
```

### Basic Usage

```bash
# Start interactive desktop UI
biomeos-ui start

# Start terminal UI
biomeos-ui --mode terminal start

# Execute a single AI command
biomeos-ui exec "deploy a biome called my-app"

# Check ecosystem status
biomeos-ui status

# List available Primals
biomeos-ui list primals

# Deploy a biome from manifest
biomeos-ui deploy my-biome.yaml --watch
```

## 🤖 AI Assistant Examples

The AI assistant understands natural language and can help you manage your entire ecosystem:

```bash
# Deployment commands
"Deploy a biome called web-app with a frontend and backend"
"Create a new biome with a database and API service"
"Deploy my-app.yaml to the production environment"

# Status and monitoring
"Show me the ecosystem status"
"What's the health of all Primals?"
"How many services are running on Toadstool?"

# Service management
"Scale the web-frontend service to 5 replicas"
"Stop the old-api service"
"Show me the logs for the database service"

# General queries
"Which Primals are available?"
"What capabilities does Songbird have?"
"How do I deploy a GPU workload?"
```

## 🔧 Configuration

### Basic Configuration

Create a `ui.yaml` configuration file:

```yaml
# UI mode (desktop, terminal, web)
ui_mode: "desktop"
theme: "dark"

# Primal endpoints
api_endpoints:
  songbird: "http://localhost:8080"
  nestgate: "http://localhost:8082"
  toadstool: "http://localhost:8084"
  beardog: "http://localhost:9000"
  # Add your custom Primals here
  my_custom_primal: "http://localhost:9999"

# AI configuration
ai_config:
  provider: "local"
  model: "local-assistant"
  temperature: 0.7
```

### Advanced Configuration

```yaml
# Multi-environment setup
environments:
  development:
    api_endpoints:
      songbird: "http://dev-songbird:8080"
  production:
    api_endpoints:
      songbird: "https://songbird.company.com"

# Feature toggles
features:
  ai_assistant: true
  real_time_monitoring: true
  deployment_wizard: true

# Custom Primal integration
custom_primals:
  my_ai_primal:
    endpoint: "http://localhost:7000"
    capabilities: ["ai", "ml", "inference"]
```

## 🏗️ Architecture

### Universal API Client
The UI uses a universal API client that can communicate with any Primal:

```rust
// Automatically discovers and connects to all available Primals
let primals = api_client.discover_primals().await?;

// Universal coordination works with any Primal
let deployment = api_client.deploy_biome(manifest).await?;

// AI commands route to appropriate Primals automatically
let response = api_client.execute_ai_command(command).await?;
```

### AI-Driven Interface
The AI assistant provides natural language interaction:

```rust
// Process natural language commands
let response = ai_assistant.process_command(
    "Deploy a biome with web services", 
    &api_client
).await?;

// AI understands ecosystem context
let status = ai_assistant.get_ecosystem_context().await?;
```

### Real-Time Updates
Event streaming keeps the UI synchronized:

```rust
// Real-time events from all Primals
let events = event_stream.get_real_time_events().await?;

// Automatic UI updates
ui.update_from_events(events).await?;
```

## 🔌 Primal Integration

### Standard Primals
Works out-of-the-box with all standard biomeOS Primals:
- **Songbird**: Orchestration and coordination
- **NestGate**: Storage and infrastructure
- **Toadstool**: Compute and runtime
- **Beardog**: Security and encryption
- **Squirrel**: AI and MCP (when available)

### Custom Primals
Integrate your custom Primal in 3 steps:

1. **Implement Universal API**: Add the standard endpoints to your Primal
2. **Add Configuration**: Include your Primal in the UI configuration
3. **Start Using**: The UI automatically detects and integrates your Primal

```yaml
# Add to ui.yaml
api_endpoints:
  my_custom_primal: "http://localhost:9999"
```

Your custom Primal just needs to implement these endpoints:
- `GET /api/v1/health` - Health check
- `POST /api/v1/coordinate` - Universal coordination
- `GET /api/v1/status` - Status information
- `GET /api/v1/events` - Event stream

### Community Primals
The UI works with any community-developed or forked Primal that implements the universal API. No additional configuration needed!

## 🎯 Use Cases

### Development
- **Local Development**: Use terminal UI for fast iteration
- **Testing**: Execute AI commands to test deployments
- **Debugging**: Real-time monitoring and log viewing

### Production
- **Operations**: Desktop UI for comprehensive ecosystem management
- **Monitoring**: Real-time dashboards and alerting
- **Automation**: CLI mode for CI/CD integration

### Custom Deployments
- **Specialized Primals**: Seamlessly integrate domain-specific Primals
- **Hybrid Environments**: Mix standard and custom Primals
- **Community Extensions**: Use community-developed Primals

## 📱 UI Modes

### Desktop Mode
Rich native desktop application:
- **Native Performance**: Fast, responsive interface
- **System Integration**: Native notifications and system tray
- **Multi-Window**: Multiple views and workspaces
- **Offline Capability**: Works with cached data when network unavailable

### Terminal Mode
Powerful terminal interface:
- **Keyboard-Driven**: Full keyboard navigation
- **SSH-Friendly**: Works over SSH connections
- **Lightweight**: Minimal resource usage
- **Scriptable**: Easy to integrate with terminal workflows

### Web Mode
Browser-based interface:
- **Universal Access**: Works on any device with a browser
- **Collaborative**: Share views and dashboards
- **Responsive**: Adapts to different screen sizes
- **Easy Deployment**: Simple to deploy and maintain

### CLI Mode
Command-line interface:
- **Automation**: Perfect for scripts and CI/CD
- **Batch Operations**: Execute multiple commands
- **JSON Output**: Machine-readable output formats
- **Integration**: Easy to integrate with other tools

## 🔄 Real-Time Features

### Event Streaming
- **Unified Stream**: Events from all Primals in one place
- **Filtering**: Filter events by type, Primal, or service
- **History**: View event history and trends
- **Alerting**: Set up alerts for specific events

### Live Monitoring
- **Health Dashboards**: Real-time health of all Primals
- **Service Status**: Live status of all services
- **Resource Usage**: Monitor CPU, memory, and storage
- **Performance Metrics**: Response times and throughput

### Automatic Updates
- **Live Refresh**: UI updates automatically as ecosystem changes
- **Smart Polling**: Efficient polling based on activity
- **Push Notifications**: Real-time alerts for important events
- **Offline Sync**: Sync changes when connectivity restored

## 🛡️ Security

### Authentication
- **Primal Authentication**: Secure authentication with each Primal
- **Token Management**: Automatic token refresh and rotation
- **Multi-Factor**: Support for MFA where available
- **Session Management**: Secure session handling

### Authorization
- **Role-Based Access**: Different permissions for different users
- **Primal-Specific**: Granular permissions per Primal
- **Audit Trail**: Full audit log of all actions
- **Secure Communication**: Encrypted communication with all Primals

## 🚀 Performance

### Optimizations
- **Lazy Loading**: Load data only when needed
- **Caching**: Intelligent caching of frequently accessed data
- **Parallel Requests**: Concurrent communication with multiple Primals
- **Efficient Updates**: Minimal UI updates for better performance

### Scalability
- **Large Ecosystems**: Handles hundreds of Primals and services
- **Distributed**: Can run across multiple machines
- **Load Balancing**: Automatic load balancing across Primal instances
- **Resource Management**: Efficient memory and CPU usage

## 🧪 Testing

### Unit Tests
```bash
# Run all tests
cargo test --package biomeos-ui

# Run specific test module
cargo test --package biomeos-ui api::tests
```

### Integration Tests
```bash
# Run integration tests
cargo test --package biomeos-ui --test integration

# Run with real Primals
INTEGRATION_TEST=true cargo test --package biomeos-ui
```

### End-to-End Tests
```bash
# Run E2E tests
cargo test --package biomeos-ui --test e2e

# Run UI tests
cargo test --package biomeos-ui --test ui
```

## 🤝 Contributing

### Development Setup
1. **Clone Repository**: `git clone https://github.com/sovereignscience/biomeOS`
2. **Install Dependencies**: `cargo build`
3. **Run Tests**: `cargo test --package biomeos-ui`
4. **Start Development**: `cargo run --package biomeos-ui`

### Adding Features
1. **Create Feature Branch**: `git checkout -b feature/my-feature`
2. **Implement Feature**: Add code and tests
3. **Test Thoroughly**: Run all tests and manual testing
4. **Submit PR**: Create pull request with description

### Custom Primal Integration
1. **Implement Universal API**: Add required endpoints to your Primal
2. **Test Integration**: Verify UI can discover and use your Primal
3. **Document Usage**: Add examples and documentation
4. **Share with Community**: Submit PR or share configuration

## 📚 Documentation

### API Documentation
- **Rust Docs**: `cargo doc --package biomeos-ui --open`
- **API Reference**: Complete API documentation
- **Examples**: Code examples for common use cases

### User Guides
- **Getting Started**: Step-by-step setup guide
- **Configuration**: Complete configuration reference
- **Troubleshooting**: Common issues and solutions

### Developer Guides
- **Architecture**: Detailed architecture documentation
- **Contributing**: How to contribute to the project
- **Custom Primals**: Guide for integrating custom Primals

## 🆘 Support

### Community
- **GitHub Issues**: Report bugs and request features
- **Discussions**: Ask questions and share ideas
- **Discord**: Real-time chat with the community

### Documentation
- **Wiki**: Comprehensive documentation and guides
- **Examples**: Real-world usage examples
- **FAQ**: Frequently asked questions

### Professional Support
- **Enterprise**: Professional support available
- **Training**: Custom training and workshops
- **Consulting**: Help with custom implementations

## 📄 License

This project is licensed under the MIT OR Apache-2.0 license.

## 🙏 Acknowledgments

- **biomeOS Team**: Core development team
- **Community Contributors**: All the amazing contributors
- **Primal Developers**: Developers of all Primals in the ecosystem
- **Open Source**: Built on top of amazing open source projects

---

**Ready to manage your entire biomeOS ecosystem with AI?** 🚀

Start with: `biomeos-ui start` and ask the AI assistant: `"Show me what you can do!"` 