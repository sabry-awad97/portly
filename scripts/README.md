# Portly Installation Scripts

Automated installation and uninstallation scripts for Portly CLI tool.

## Installation Scripts

### Windows (PowerShell)

**Quick install:**
```powershell
iwr -useb https://raw.githubusercontent.com/sabry-awad97/portly/main/scripts/install.ps1 | iex
```

**With parameters:**
```powershell
# Download first
Invoke-WebRequest -Uri "https://raw.githubusercontent.com/sabry-awad97/portly/main/scripts/install.ps1" -OutFile "install.ps1"

# Install latest version
.\install.ps1

# Install specific version
.\install.ps1 -Version "0.1.0"

# Install to custom directory
.\install.ps1 -InstallDir "C:\Tools\portly"
```

**Features:**
- Detects system architecture (x64, ARM64)
- Downloads from GitHub Releases API
- Installs to `%LOCALAPPDATA%\portly\bin`
- Adds to user PATH automatically
- Verifies installation with `portly --version`
- Comprehensive error handling

### Unix-like (bash)

**Quick install:**
```bash
curl -sSL https://raw.githubusercontent.com/sabry-awad97/portly/main/scripts/install.sh | bash
```

**With parameters:**
```bash
# Install latest version
./install.sh

# Install specific version
./install.sh 0.1.0

# Install to custom directory
PORTLY_INSTALL_DIR=/opt/portly ./install.sh
```

**Features:**
- Detects OS (Linux, macOS) and architecture
- Downloads from GitHub Releases API
- Installs to `~/.local/bin`
- Makes binary executable
- Checks if directory is in PATH
- Comprehensive error handling

## Uninstallation Scripts

### Windows (PowerShell)

**Quick uninstall:**
```powershell
iwr -useb https://raw.githubusercontent.com/sabry-awad97/portly/main/scripts/uninstall.ps1 | iex
```

**With parameters:**
```powershell
# Interactive uninstall (with confirmation)
.\uninstall.ps1

# Force uninstall (skip confirmation)
.\uninstall.ps1 -Force

# Also remove configuration files
.\uninstall.ps1 -RemoveConfig

# Force uninstall and remove config
.\uninstall.ps1 -Force -RemoveConfig
```

**What it does:**
- Removes binary from `%LOCALAPPDATA%\portly\bin`
- Cleans up PATH environment variable
- Optionally removes config from `%APPDATA%\portly`
- Confirmation prompts (unless `-Force`)

### Unix-like (bash)

**Quick uninstall:**
```bash
curl -sSL https://raw.githubusercontent.com/sabry-awad97/portly/main/scripts/uninstall.sh | bash
```

**With parameters:**
```bash
# Interactive uninstall (with confirmation)
./uninstall.sh

# Force uninstall (skip confirmation)
./uninstall.sh --force

# Also remove configuration files
./uninstall.sh --remove-config

# Force uninstall and remove config
./uninstall.sh --force --remove-config
```

**What it does:**
- Removes binary from `~/.local/bin`
- Optionally removes config from `~/.config/portly`
- Confirmation prompts (unless `--force`)
- Provides instructions for PATH cleanup

## Requirements

### Windows
- PowerShell 5.1 or later
- Internet connection
- Windows 10 or later

### Unix-like
- bash
- curl
- tar
- Internet connection

## Troubleshooting

### Windows

**Error: "Execution of scripts is disabled on this system"**
```powershell
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
```

**Error: "Failed to fetch release information"**
- Check internet connection
- Verify GitHub is accessible
- Check if behind a proxy (set `$env:HTTP_PROXY`)

**Binary not in PATH after installation**
- Restart your terminal
- Or manually add: `$env:PATH += ";$env:LOCALAPPDATA\portly\bin"`

### Unix-like

**Error: "curl: command not found"**
```bash
# Ubuntu/Debian
sudo apt-get install curl

# macOS
brew install curl
```

**Error: "Permission denied"**
```bash
# Make script executable
chmod +x install.sh

# Or use bash directly
bash install.sh
```

**Binary not in PATH after installation**
Add to your shell profile (`~/.bashrc`, `~/.zshrc`, etc.):
```bash
export PATH="$PATH:$HOME/.local/bin"
```

## Testing

Test scripts are located in `tests/scripts/`:

```powershell
# Test PowerShell install script
.\tests\scripts\test_install_ps1.ps1

# Test bash install script
.\tests\scripts\test_install_sh.ps1

# Test uninstall scripts
.\tests\scripts\test_uninstall.ps1

# Run all tests
.\tests\scripts\test_install_ps1.ps1
.\tests\scripts\test_install_sh.ps1
.\tests\scripts\test_uninstall.ps1
```

## Development

### Adding New Features

1. Write tests first (RED phase)
2. Implement feature (GREEN phase)
3. Refactor if needed (REFACTOR phase)
4. Update documentation

### Best Practices

- Use GitHub Releases API (not hardcoded URLs)
- Provide verbose output with colors
- Handle errors gracefully with helpful messages
- Support both interactive and non-interactive modes
- Make scripts idempotent (safe to run multiple times)
- Minimize dependencies

## Security

**Important:** These scripts download and execute code from the internet. Always:
- Review scripts before running
- Use HTTPS URLs only
- Verify the source repository
- Check script signatures if available

For maximum security, download and inspect scripts before running:

```powershell
# Windows
Invoke-WebRequest -Uri "https://raw.githubusercontent.com/sabry-awad97/portly/main/scripts/install.ps1" -OutFile "install.ps1"
# Review install.ps1
.\install.ps1
```

```bash
# Unix
curl -O https://raw.githubusercontent.com/sabry-awad97/portly/main/scripts/install.sh
# Review install.sh
bash install.sh
```

## License

MIT License - see [LICENSE](../LICENSE) file for details

## Contributing

Contributions welcome! Please:
1. Follow existing script patterns
2. Add tests for new features
3. Update documentation
4. Test on multiple platforms

---

**Repository:** https://github.com/sabry-awad97/portly
