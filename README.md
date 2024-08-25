
![SHAD3](https://github.com/styromaniac/shad3/raw/main/SHAD3.png?v=1)

# SHAD3

shad3 is an application using SHA3-512 to hash entries in a list or multiple lists.

For multiple lists, simply provide the highest numbered file's location/URL.

Use `--expect "prefix goes here"` after the file location/URL to only hash texts following the given prefix.

Use `--output my-file-path.txt` to place the checksums file somewhere besides checksums/checksums.txt.

## Example:

### Android, Linux, and MacOS:
`bash
./shad3 http://blocklists.io/block04.txt --expect "127.0.0.1 " --output var/www/html/pornSites.txt
`

### Windows:
`bash
shad3.exe http://blocklists.io/block04.txt --expect "127.0.0.1 " --output Documents\pornQueries.txt
`

## Installation

### From Cargo
1. Ensure you have Rust installed on your system. If not, follow the instructions [here](https://www.rust-lang.org/tools/install).
   
2. Install SHAD3 via Cargo by running:
   ```bash
   cargo install shad3
   ```

3. Once installed, you can use SHAD3 from the command line:
   ```bash
   shad3 --help
   ```

### From Source
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
   shad3 --help
   ```

### Android
rustup isn't available to make your life easier, but the command below is, though you are required to install F-Droid or (I recommend for automatic updates) F-Droid Basic, then through either, install Termux, open it, paste the command, then hit Enter. DO NOT INSTALL TERMUX FROM THE PLAY STORE AS IT IS FUNCTIONALLY USELESS.
```bash
pkg update && pkg upgrade && pkg install -y rust git build-essential && git clone https://github.com/styromaniac/shad3.git && cd shad3 && cargo build --release && cp target/release/shad3 $PREFIX/bin/ && echo -e '
# Rust and shad3 environment setup
export PATH=$PATH:/data/data/com.termux/files/home/.cargo/bin:$PREFIX/bin
export TMPDIR=/data/data/com.termux/files/home/temp
mkdir -p $TMPDIR

# Alias for updating and upgrading packages
alias pkgup="pkg update && pkg upgrade"

# Function to update shad3
update_shad3() {
    cd ~/shad3 && git pull && cargo build --release && cp target/release/shad3 $PREFIX/bin/ && echo "shad3 updated successfully."
}

# Alias for updating shad3
alias update-shad3="update_shad3"' >> ~/.bashrc && source ~/.bashrc && shad3 --help
```

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
   shad3 --help
   ```

### Windows
1. Install Rust from https://www.rust-lang.org/tools/install

2. Open Command Prompt or PowerShell and run:
   ```bash
   git clone https://github.com/styromaniac/shad3.git
   cd shad3
   cargo build --release
   ```

3. The executable will be in `target\release\shad3.exe`. You can move it to a directory in your PATH or run it from its current location.

4. Verify the installation by running:
   ```bash
   shad3 --help
   ```
