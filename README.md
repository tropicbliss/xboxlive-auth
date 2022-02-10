# xboxlive-auth
A proof-of-concept program to retrieve a Minecraft account's bearer token based on the new Microsoft authentication scheme.

This program will not work on macOS because of TLS renegotiation issues.

## Compiling from source

1. Download the `rustup` toolchain right [here](https://rustup.rs/). Follow the instructions for your platform.
2. Run `git clone https://github.com/tropicbliss/xboxlive-auth.git` in an appropriate directory to clone the repo.
3. In the folder named `xboxlive-auth`, run `cargo build --release`. The resulting executable file after compilation should be in the `target/release/` directory relative from the `xboxlive-auth` folder.
