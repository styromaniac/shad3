![SHAD3](https://github.com/styromaniac/shad3/raw/main/SHAD3.png)

# SHAD3

shad3 is an application using SHA3-512 to hash entries in a list or multiple lists.

For multiple lists, simply provide the highest numbered file's location/URL.

Use --expect "prefix goes here" after the file location/URL to only hash texts following the given prefix.

## Example:

### Android, Linux, and MacOS:
./shad3 http://blocklists.io/block04.txt --expect "127.0.0.1 "

### Windows:
shad3.exe http://blocklists.io/block04.txt --expect "127.0.0.1 "

## Android Installation
rustup isn't available to make your life easier, but the command below is, though you are required to install F-Droid or (I recommend for automatic updates) F-Droid Basic, then through either, install Termux, open it, paste the command, then hit Enter. DO NOT INSTALL TERMUX FROM THE PLAY STORE AS IT IS FUNCTIONALLY USELESS.
```
pkg update && pkg upgrade && pkg install -y rust git build-essential && git clone https://github.com/styromaniac/SHAD3.git && cd SHAD3 && cargo build --release && cp target/release/shad3 $PREFIX/bin/ && echo -e '\n# Rust and shad3 environment setup\nexport PATH=$PATH:/data/data/com.termux/files/home/.cargo/bin:$PREFIX/bin\nexport TMPDIR=/data/data/com.termux/files/home/temp\nmkdir -p $TMPDIR\n\n# Alias for updating and upgrading packages\nalias pkgup="pkg update && pkg upgrade"\n\n# Function to update shad3\nupdate_shad3() {\n    cd ~/SHAD3 && git pull && cargo build --release && cp target/release/shad3 $PREFIX/bin/ && echo "shad3 updated successfully."\n}\n\n# Alias for updating shad3\nalias update-shad3="update_shad3"' >> ~/.bashrc && source ~/.bashrc && shad3 --help
```
