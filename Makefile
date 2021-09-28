
.PHONY: loader
loader:
	cd loader && cargo build --release

.PHONY: kernel
kernel:
	cd kernel && cargo build --release

run: loader kernel
	./run_qemu.sh --serial

.PHONY: test
test:
	cd kernel && cargo test --release -- --serial
