# SHAD3

<img src='./SHAD3.png' alt='SHAD3 logo' style='max-width: 100%; height: auto;'>

SHAD3 is an application using SHA3-512 to hash entries in a list or multiple lists, typically those made for Pi-hole. This is designed for [SHATTER](https://addons.mozilla.org/en-US/firefox/addon/shatt3r/) and similar applications or addons. Ideally, it serves to maintain the confidentiality of blocked content, which may include exploitative or abusive websites.

## Example Usage
```bash
shad3 <blocklist-url-or-path> [output-path]
```

## Installation Options with Architecture Detection

### Binary Installation (Recommended)
Pre-built binaries are available in the [releases](https://github.com/styromaniac/shad3/releases) section. The following one-liners automatically detect the CPU architecture and download the correct binary:

#### Linux Binary
```bash
ARCH=$(uname -m) && case $ARCH in \
  'x86_64') BIN=shad3-linux_x86_64.tar.gz ;; \
  'aarch64') BIN=shad3-linux_aarch64.tar.gz ;; \
  *) echo 'Unsupported architecture: $ARCH' && exit 1 ;; \
esac && curl -LO https://github.com/styromaniac/shad3/releases/latest/download/$BIN && \
tar -xzf $BIN && sudo mv shad3 /usr/local/bin && cd .. && rm -rf $(basename $PWD)
```

#### macOS Binary
```bash
ARCH=$(uname -m) && case $ARCH in \
  'x86_64') BIN=shad3-macos_x86_64.tar.gz ;; \
  'aarch64') BIN=shad3-macos_aarch64.tar.gz ;; \
  *) echo 'Unsupported architecture: $ARCH' && exit 1 ;; \
esac && curl -LO https://github.com/styromaniac/shad3/releases/latest/download/$BIN && \
tar -xzf $BIN && sudo mv shad3 /usr/local/bin && cd .. && rm -rf $(basename $PWD)
```

#### Windows Binary (PowerShell)
```powershell
$ARCH = (Get-WmiObject Win32_Processor).Architecture; \
if ($ARCH -eq 9) { $BIN = 'shad3-windows_x86_64.zip' } elseif ($ARCH -eq 5) { $BIN = 'shad3-windows_aarch64.zip' } \
else { Write-Host 'Unsupported architecture: $ARCH' -ForegroundColor Red; exit } \
Invoke-WebRequest -Uri https://github.com/styromaniac/shad3/releases/latest/download/$BIN -OutFile $BIN; \
Expand-Archive -Path $BIN -DestinationPath .; Move-Item -Path .\shad3.exe -Destination $env:ProgramFiles\shad3.exe; \
cd ..; Remove-Item -Recurse -Force (Get-Location)
```

#### Termux Binary
```bash
ARCH=$(uname -m) && case $ARCH in \
  'x86_64') BIN=shad3-termux_x86_64.zip ;; \
  'aarch64') BIN=shad3-termux_aarch64.zip ;; \
  *) echo 'Unsupported architecture: $ARCH' && exit 1 ;; \
esac && curl -LO https://github.com/styromaniac/shad3/releases/latest/download/$BIN && \
unzip $BIN && mv shad3 ~/../usr/bin && cd .. && rm -rf $(basename $PWD)
```

### Installation from Source

If you'd like to compile SHAD3 from source, use the following one-liners for each environment. The source code is dynamically fetched from the latest release:

#### Linux Source (with package manager detection)
```bash
detect_pkg_manager() { \
  if command -v apt-get &>/dev/null; then \
    sudo apt-get update && sudo apt-get install -y build-essential ;; \
  elif command -v dnf &>/dev/null; then \
    sudo dnf groupinstall 'Development Tools' ;; \
  elif command -v pacman &>/dev/null; then \
    sudo pacman -Sy --noconfirm base-devel ;; \
  elif command -v zypper &>/dev/null; then \
    sudo zypper install -t pattern devel_basis ;; \
  else \
    echo 'Unsupported distribution. Please install the required development tools manually.'; exit 1 ;; \
  fi \
} \
detect_pkg_manager && LATEST_RELEASE=$(curl -s https://api.github.com/repos/styromaniac/shad3/releases/latest | \
grep tarball_url | cut -d '"' -f 4) && curl -L $LATEST_RELEASE | tar -xz && cd shad3-* && \
cargo build --release && sudo mv target/release/shad3 /usr/local/bin && cd .. && rm -rf shad3-* 
```

#### macOS Source
```bash
LATEST_RELEASE=$(curl -s https://api.github.com/repos/styromaniac/shad3/releases/latest | \
grep tarball_url | cut -d '"' -f 4) && curl -L $LATEST_RELEASE | tar -xz && cd shad3-* && \
cargo build --release && sudo mv target/release/shad3 /usr/local/bin && cd .. && rm -rf shad3-* 
```

#### Windows Source (PowerShell)
```powershell
$LATEST_RELEASE = Invoke-RestMethod https://api.github.com/repos/styromaniac/shad3/releases/latest | \
ForEach-Object { $_.zipball_url }; \
Invoke-WebRequest -Uri $LATEST_RELEASE -OutFile latest.zip; Expand-Archive -Path latest.zip -DestinationPath .; \
cd shad3-*; cargo build --release; Move-Item -Path .\target\release\shad3.exe -Destination $env:ProgramFiles\shad3.exe; \
cd ..; Remove-Item -Recurse -Force .\shad3-* latest.zip
```

#### Termux Source
```bash
pkg install -y rust && LATEST_RELEASE=$(curl -s https://api.github.com/repos/styromaniac/shad3/releases/latest | \
grep tarball_url | cut -d '"' -f 4) && curl -L $LATEST_RELEASE | tar -xz && cd shad3-* && \
cargo build --release && mv target/release/shad3 ~/../usr/bin && cd .. && rm -rf shad3-* 
```

### Verification
After installation, verify with:
```bash
shad3
```

## Platform Compatibility

SHAD3 is compatible with the following:
- **Operating Systems**: Linux, macOS, Windows, Termux (Android/ChromeOS)
- **Architectures**: x86-64, ARM64

## Troubleshooting

If you encounter any issues during installation or use, please check:
- Ensure all dependencies and build tools are installed (see above for platform-specific tools).
- Verify the binary is added to your PATH if using a direct download.
- For cargo-related issues, consult the [Rust documentation](https://doc.rust-lang.org/cargo/).

