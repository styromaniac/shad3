# SHAD3

<img src="https://raw.githubusercontent.com/styromaniac/shad3/refs/heads/main/SHAD3.png" alt="SHAD3 logo" style="max-width: 100%; height: auto;">

SHAD3 is an application using SHA3-512 to hash entries in a list or multiple lists, typically those made for Pi-hole. This is designed for [SHATTER](https://addons.mozilla.org/en-US/firefox/addon/shatt3r/) and similar applications or addons. Ideally you would use this for keeping secret what is being blocked. Law enforcement could use this tool for websites containing exploitative and abusive content.

Set a custom path after the first argument to save the output file anywhere you need to.

## Example:
```bash
shad3 <blocklist-url-or-path> [output-path]
```

## Installation

### From Binary (Recommended)
Pre-built binaries are available in the [releases](https://github.com/styromaniac/shad3/releases) section. You can download the appropriate archive for your operating system, extract it, and start using SHAD3 without the need to build from source.

1. Download the binary release for your platform from the [releases](https://github.com/styromaniac/shad3/releases) section.
2. Extract the contents of the archive.
3. Move the binary to a directory in your PATH (optional).
4. Verify the installation by running:
    ```bash
    shad3
    ```

### From Cargo
1. Ensure you have Rust installed on your system. If not, follow the instructions [here](https://www.rust-lang.org/tools/install).
2. Install SHAD3 via Cargo by running:
    ```bash
    cargo install shad3
    ```
3. Once installed, you can use SHAD3 from the command line:
    ```bash
    shad3
    ```

### From Source (Optional)
If you'd prefer to build SHAD3 from source, follow the instructions below:

1. Ensure you have Rust installed on your system. If not, follow the instructions [here](https://www.rust-lang.org/tools/install).
2. Clone the repository and build:
    ```bash
    git clone https://github.com/styromaniac/shad3.git
    cd shad3
    cargo build --release
    ```
3. Move the binary to a directory in your PATH:
    ```bash
    sudo mv target/release/shad3 /usr/local/bin/
    ```
4. Verify the installation by running:
    ```bash
    shad3
    ```

### Termux
rustup isn't available to make your life easier, but the command below is. You are required to install F-Droid or (recommended for automatic updates) F-Droid Basic. Then, through either, install Termux. Open it, paste the command, and hit Enter.

```bash
pkg update && pkg upgrade && pkg install -y wget tar rust git build-essential && wget -O shad3-src.tar.gz $(curl -s https://api.github.com/repos/styromaniac/shad3/releases/latest | grep "tarball_url" | cut -d '"' -f 4) && tar -xzvf shad3-src.tar.gz && cd styromaniac-shad3-* && cargo build --release && mv target/release/shad3 $PREFIX/bin/ && echo -e '# Rust and shad3 environment setup\nexport PATH=$PATH:/data/data/com.termux/files/home/.cargo/bin:$PREFIX/bin\nexport TMPDIR=/data/data/com.termux/files/home/temp\nmkdir -p $TMPDIR\n# Alias for updating and upgrading packages\nalias pkgup="pkg update && pkg upgrade"\n# Function to update shad3\nupdate_shad3() { cd ~/styromaniac-shad3-* && git pull && cargo build --release && cp target/release/shad3 $PREFIX/bin/; echo "shad3 updated successfully."; }\n# Alias for updating shad3\nalias update-shad3="update_shad3"' >> ~/.bashrc && source ~/.bashrc && shad3
```
**Note:** Do **NOT** install Termux from the Play Store as it lacks necessary functionalities.

### Linux and macOS
1. Install Rust if you haven't already:
    ```bash
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    ```
2. Clone the repository and build:
    ```bash
    git clone https://github.com/styromaniac/shad3.git
    cd shad3
    cargo build --release
    ```
3. Move the binary to a directory in your PATH:
    ```bash
    sudo mv target/release/shad3 /usr/local/bin/
    ```
4. Verify the installation by running:
    ```bash
    shad3
    ```

### Windows
1. Install Rust from [Rust's official website](https://www.rust-lang.org/tools/install).
2. Open Command Prompt or PowerShell and run:
    ```bash
    git clone https://github.com/styromaniac/shad3.git
    cd shad3
    cargo build --release
    ```
3. The executable will be in `target\release\shad3.exe`. You can move it to a directory in your PATH or run it from its current location.
4. Verify the installation by running:
    ```bash
    shad3
    ```

## Contributing

Contributions are welcome! Please fork the repository and submit a pull request for any enhancements or bug fixes.

## License

This project is licensed under the [GNU General Public License v3.0](LICENSE).

## Support

For support or inquiries, please open an issue on the [GitHub repository](https://github.com/styromaniac/shad3/issues).
