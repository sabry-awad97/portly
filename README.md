# Portly

> Modern CLI tool for managing local development ports on Windows

[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

Portly helps you understand what's running on your ports with beautiful tables, framework detection, Docker integration, and interactive process management.

## Features

- 🔍 **Port Scanning** - List all listening ports with process information
- 🎯 **Framework Detection** - Automatically detect Next.js, Rust, Python, Docker, and more
- 🐳 **Docker Integration** - Identify Docker containers and their frameworks
- 📊 **Process Management** - Kill processes by port or PID, clean orphaned processes
- 👀 **Watch Mode** - Real-time monitoring of port changes
- 📝 **JSON Output** - Machine-readable output for scripting
- ⚙️ **Configuration** - Customize filters, colors, and defaults

## Installation

### From Source

```bash
git clone https://github.com/portly/portly
cd portly
cargo build --release
```

The binary will be at `target/release/portly.exe`

### Add to PATH (Optional)

```powershell
# Add to your PowerShell profile
$env:PATH += ";C:\path\to\portly\target\release"
```

## Quick Start

```bash
# List all listening ports
portly

# Show detailed information for a specific port
portly details 3000

# Kill a process by port
portly kill 3000

# Watch for port changes in real-time
portly watch

# List all dev processes (not just port-bound)
portly ps
```

## Commands

### `portly` or `portly list`

List all listening ports with process information.

```bash
portly
portly list
portly list --all      # Include system processes
portly list --json     # JSON output
```

**Example output:**
```
PORT   PROCESS          PID    FRAMEWORK    PROJECT
3000   node.exe         12345  Next.js      my-app
5432   postgres.exe     5678   PostgreSQL   [Docker]
8080   rust.exe         9012   Rust         portly
```

### `portly details <port>`

Show detailed information about a specific port.

```bash
portly details 3000
portly details 3000 --no-prompt  # Skip kill confirmation
portly details 3000 --json
```

**Shows:**
- Process name and PID
- Command line
- Memory usage
- Uptime
- Working directory
- Git branch (if in a git repo)
- Process tree
- Interactive kill prompt

### `portly kill <targets...>`

Kill processes by port or PID.

```bash
portly kill 3000           # Kill process on port 3000
portly kill 3000 5000      # Kill multiple ports
portly kill 12345          # Kill by PID
portly kill 3000 -f        # Force kill (SIGKILL)
```

### `portly clean`

Find and kill orphaned/zombie processes.

```bash
portly clean              # Dry-run (show what would be killed)
portly clean --execute    # Actually kill orphaned processes
```

### `portly ps`

List all development processes (not just port-bound).

```bash
portly ps
portly ps --all     # Include system processes
portly ps --json
```

**Shows:**
- All Node.js, Python, Rust, Go processes
- Docker containers
- CPU and memory usage
- Color-coded by CPU usage (red >25%, yellow >5%, green ≤5%)

### `portly watch`

Watch for port changes in real-time.

```bash
portly watch
portly watch -i 5    # Update every 5 seconds (default: 2)
```

**Shows:**
- Timestamp for each change
- NEW ports (green)
- CLOSED ports (red)
- Press Ctrl+C to stop

### `portly config`

Manage configuration.

```bash
portly config init    # Create default config file
portly config path    # Show config file location
portly config reset   # Reset to defaults
```

## Configuration

Config file location: `%APPDATA%\portly\config.toml`

### Default Configuration

```toml
[display]
colors = true
compact = false

[filters]
exclude_system = true
exclude_processes = [
    "Spotify",
    "Chrome",
    "Firefox",
    "Slack",
    "Discord",
    "Code",
    "Teams",
]

[defaults]
show_all = false
json_output = false
```

### Customization

Edit the config file to:
- Exclude specific processes from listings
- Disable colors for piping to files
- Change default flags

## Global Flags

These flags work with all commands:

- `--json` - Output in JSON format
- `--all` - Show all processes (including system)
- `--no-color` - Disable colored output

## Framework Detection

Portly automatically detects:

- **Next.js** - `next dev`, `next start`
- **Vite** - `vite`, `vite dev`
- **Rust** - `cargo run`, `cargo watch`
- **Python** - `python`, `uvicorn`, `flask`, `django`
- **Go** - `go run`
- **Docker** - PostgreSQL, Redis, nginx, MongoDB, MySQL, RabbitMQ, Elasticsearch

## Docker Integration

Portly integrates with Docker to:
- Identify containers by port
- Detect framework from Docker image
- Group Docker processes in `ps` output

**Graceful degradation**: Works without Docker installed.

## Examples

### Find what's using port 3000
```bash
portly details 3000
```

### Kill all Node.js processes on specific ports
```bash
portly kill 3000 3001 3002
```

### Monitor ports during development
```bash
portly watch
```

### Export port list to JSON
```bash
portly list --json > ports.json
```

### Find and clean orphaned processes
```bash
portly clean --execute
```

### See all dev processes with CPU usage
```bash
portly ps
```

## Development

### Prerequisites

- Rust 1.85+ (2024 Edition)
- Windows 10+

### Build

```bash
cargo build
```

### Test

```bash
cargo test
```

### Lint

```bash
cargo clippy -- -W clippy::all
```

### Format

```bash
cargo fmt
```

## Architecture

Portly follows Clean Architecture principles:

- **Platform Abstraction** - Windows-specific code isolated in `platform/` module
- **Deep Modules** - Small interfaces, deep implementations
- **TDD** - All features built with test-driven development
- **No Unsafe** - Pure safe Rust throughout

### Project Structure

```
portly/
├── src/
│   ├── main.rs           # Entry point and command handlers
│   ├── cli.rs            # Clap command definitions
│   ├── scanner.rs        # Port scanning orchestration
│   ├── process.rs        # Process information types
│   ├── framework.rs      # Framework detection
│   ├── display.rs        # Table/JSON output
│   ├── details.rs        # Port details view
│   ├── config.rs         # Configuration management
│   ├── docker.rs         # Docker integration
│   ├── error.rs          # Error types
│   └── platform/         # Platform abstraction
│       ├── mod.rs        # Platform trait
│       └── windows.rs    # Windows implementation
└── tests/                # Integration tests
```

## Contributing

Contributions welcome! Please:

1. Follow Rust conventions
2. Write tests for new features
3. Run `cargo clippy` and `cargo fmt`
4. Update documentation

## Roadmap

- [ ] Cross-platform support (macOS, Linux)
- [ ] Process filtering by name/pattern
- [ ] Port history tracking
- [ ] Export to various formats (CSV, HTML)
- [ ] Web UI for monitoring

## License

MIT License - see [LICENSE](LICENSE) file for details

## Acknowledgments

Built with:
- [clap](https://github.com/clap-rs/clap) - Command line parsing
- [tabled](https://github.com/zhiburt/tabled) - Table formatting
- [sysinfo](https://github.com/GuillaumeGomez/sysinfo) - System information
- [netstat2](https://github.com/zhongzc/netstat2) - Network socket information
- [colored](https://github.com/mackwic/colored) - Terminal colors

---

**Made with ❤️ for developers who need to manage their ports**
