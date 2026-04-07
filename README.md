# Portly

> Modern CLI tool for managing local development ports on Windows

[![CI](https://github.com/portly/portly/workflows/CI/badge.svg)](https://github.com/portly/portly/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

**Portly** helps developers understand what's running on their ports with beautiful tables, framework detection, and interactive process management.

## Features

- 🔍 **Instant visibility** - See all listening ports with process details
- 🎨 **Beautiful output** - Colored tables with status indicators
- 🔧 **Framework detection** - Automatically identifies Next.js, Vite, Django, Rust, Go, Docker
- ⚡ **Interactive management** - Kill processes by port or PID with confirmation
- 📊 **Process listing** - View all dev processes with CPU and memory usage
- 🐳 **Docker integration** - Identify and manage Docker containers
- 📝 **JSON export** - Machine-readable output for scripting
- ⚙️ **Configuration** - Customize filters, colors, and behavior

## Installation

### From Source

```bash
git clone https://github.com/portly/portly.git
cd portly
cargo build --release
```

The binary will be at `target/release/portly.exe`

### From Cargo (Coming Soon)

```bash
cargo install portly
```

## Usage

### List all listening ports

```bash
portly
# or
portly list
```

### Show detailed information about a port

```bash
portly details 3000
```

### Kill a process by port

```bash
portly kill 3000
```

### List all dev processes

```bash
portly ps
```

### Watch for port changes

```bash
portly watch
```

### JSON output

```bash
portly --json
portly details 3000 --json
```

## Commands

| Command | Description |
|---------|-------------|
| `portly` or `portly list` | List all listening ports |
| `portly details <port>` | Show detailed info about a port |
| `portly kill <port\|pid>` | Kill process by port or PID |
| `portly clean` | Find and kill orphaned processes |
| `portly ps` | List all dev processes |
| `portly watch` | Watch for port changes in real-time |

## Global Flags

| Flag | Description |
|------|-------------|
| `--json` | Output in JSON format |
| `--all` | Show all processes including system apps |
| `--no-color` | Disable colored output |

## Development

### Prerequisites

- Rust 1.85+ (2024 Edition)
- Windows 10+ (macOS and Linux support coming soon)

### Build

```bash
# Check compilation
cargo check

# Build debug
cargo build

# Build release
cargo build --release
```

### Test

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture
```

### Lint

```bash
# Run clippy
cargo clippy -- -W clippy::all

# Format code
cargo fmt
```

## Configuration

Portly can be configured via `%APPDATA%\portly\config.toml`:

```toml
[display]
colors = true
compact = false

[filters]
exclude_system = true
exclude_processes = ["Spotify", "Chrome", "Slack"]

[defaults]
show_all = false
json_output = false
```

## Project Status

Portly is under active development. Current milestone: **v0.1.0 (MVP)**

See [issues/](issues/) for implementation roadmap.

## Contributing

Contributions are welcome!.

## License

MIT License - see [LICENSE](LICENSE) for details

## Acknowledgments

- Built with [clap](https://github.com/clap-rs/clap), [tabled](https://github.com/zhiburt/tabled), and [sysinfo](https://github.com/GuillaumeGomez/sysinfo)

---

**Tagline**: "Know your ports, love your ports"
