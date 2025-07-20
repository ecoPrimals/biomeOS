# game-engine

Lightweight game engine for BiomeOS

## Features

- Gaming
- Custom { name: "GameLogic", description: "Game logic processing" }

## Installation

```bash
cargo build --release
```

## Usage

```bash
# Run the primal
cargo run

# Run with verbose logging
cargo run -- --verbose

# Run tests
cargo test

# Run example
cargo run --example basic_usage
```

## API

### Health Check

```bash
curl http://localhost:8080/health
```

### Send Request

```bash
curl -X POST http://localhost:8080/api/request \
  -H "Content-Type: application/json" \
  -d '{"method": "ping", "payload": {}}'
```

## Configuration

The primal can be configured through environment variables or configuration files.

## Development

This primal was generated using the BiomeOS Primal SDK.

### Running Tests

```bash
cargo test
cargo test --doc
```

### Building for Release

```bash
cargo build --release
```

## License

This project is licensed under the MIT License.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
