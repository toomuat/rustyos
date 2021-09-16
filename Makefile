
.PHONY: loader
loader:
	cd loader && cargo build --release

run: loader
	./run_qemu.sh --serial
