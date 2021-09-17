## rustyos

<br>

```
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

rustup component add rust-src --toolchain nightly-x86_64-unknown-linux-gnu

# Clone rustyos
git clone https://github.com/toomuat/rustyos
cd rustyos
git submodule update --init --recursive

or

git clone --recurse-submodules https://github.com/toomuat/rustyos

# Build EDK2

# Ubuntu 18.04
sudo apt install -y git python3 build-essential uuid-dev wget unzip nasm iasl curl qemu-system-x86

# CentOS 8
sudo dnf group install 'Development Tools'
sudo dnf install -y iasl libuuid-devel
sudo dnf --enablerepo=powertools install nasm

cd edk2
cd BaseTools/Source/C/BrotliCompress/brotli
wget https://github.com/tianocore/edk2/releases/download/edk2-stable202011/submodule-BaseTools-Source-C-BrotliCompress-brotli.zip
unzip submodule-BaseTools-Source-C-BrotliCompress-brotli.zip
cd -
make -C BaseTools
. ./edksetup.sh
build -a X64 -t GCC5 -p OvmfPkg/OvmfPkgX64.dsc
ls Build/OvmfX64/DEBUG_GCC5/FV/OVMF*
Build/OvmfX64/DEBUG_GCC5/FV/OVMF.fd  Build/OvmfX64/DEBUG_GCC5/FV/OVMF_CODE.fd  Build/OvmfX64/DEBUG_GCC5/FV/OVMF_VARS.fd

# Bulid loader
make loader

# Run loader and kernel on QEMU
make run

or

./run_qemu.sh --monitor     # Enable QEMU monitor
./run_qemu.sh --serial      # Output to serial console
```

