
set -e

# Build EDK2

<<'###'
# Ubuntu 18.04
sudo apt install -y git python3 build-essential uuid-dev wget unzip nasm iasl curl qemu-system-x86

# CentOS 8
sudo dnf group install 'Development Tools'
sudo dnf install -y iasl libuuid-devel
sudo dnf --enablerepo=powertools install nasm
###

exit 0
cd edk2
cd BaseTools/Source/C/BrotliCompress/brotli
wget https://github.com/tianocore/edk2/releases/download/edk2-stable202011/submodule-BaseTools-Source-C-BrotliCompress-brotli.zip
unzip submodule-BaseTools-Source-C-BrotliCompress-brotli.zip
cd -
make -C BaseTools
. ./edksetup.sh
build -a X64 -t GCC5 -p OvmfPkg/OvmfPkgX64.dsc
ls Build/OvmfX64/DEBUG_GCC5/FV/OVMF*
