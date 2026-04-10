# Changelog

All notable changes to Portly will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed
- Refactored process killing to use sysinfo's native cross-platform `kill_with()` method instead of platform-specific commands (e.g., taskkill on Windows)
- Improved kill performance by eliminating process spawning overhead (50-100ms reduction)
- Enhanced error messages for kill operations with better signal support detection

## [0.1.0] - 2026-04-07

### Added

#### Core Features
- Port scanning with process information display
- Framework detection (Next.js, Vite, Rust, Python, Go, Docker)
- Docker integration with container identification
- Process management (kill by port/PID, clean orphaned processes)
- Real-time watch mode for port monitoring
- JSON output for all commands
- TOML configuration file support

#### Commands
- `portly list` - List all listening ports
- `portly details <port>` - Show detailed port information
- `portly kill <targets...>` - Kill processes by port or PID
- `portly clean` - Find and kill orphaned processes
- `portly ps` - List all development processes
- `portly watch` - Real-time port monitoring
- `portly config` - Configuration management (init, path, reset)

#### Features
- Beautiful table output with colors
- Process tree visualization
- Git branch detection in working directories
- CPU and memory usage display
- Configurable process filters
- Interactive kill prompts
- Graceful Docker degradation

### Technical
- Built with Rust 1.85+ (2024 Edition)
- Clean Architecture with platform abstraction
- Test-driven development (46 passing tests)
- Zero unsafe code
- Comprehensive error handling
- Cross-platform implementation (Windows, macOS, Linux)

### Dependencies
- clap 4.6.0 - CLI parsing
- tabled 0.20.0 - Table formatting
- sysinfo 0.38.4 - System information
- netstat2 0.11.2 - Network socket information
- colored 3.1.1 - Terminal colors
- serde 1.0.228 - Serialization
- anyhow 1.0.102 - Error handling

[0.1.0]: https://github.com/portly/portly/releases/tag/v0.1.0
