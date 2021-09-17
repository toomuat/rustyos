## rustyos

<br>

```
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

rustup component add rust-src --toolchain nightly-x86_64-unknown-linux-gnu

# Bulid
make loader

# Run
make run

./run_qemu.sh --monitor     # Enable QEMU monitor
./run_qemu.sh --serial      # Output to serial console
```

