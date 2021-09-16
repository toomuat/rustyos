
.PHONY: loader
loader:
	cd loader && cargo build --release

.PHONY: kernel
kernel:
	cd kernel && cargo build --release

run: loader
	./run_qemu.sh --serial
