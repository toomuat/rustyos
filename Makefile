
.PHONY: loader
loader:
	cd loader && cargo build --release

.PHONY: kernel
kernel:
	cd kernel && cargo build --release

run: loader kernel
	./run_qemu.sh --cui

all:
	cd loader && cargo build --release && cd -
	cd kernel && cargo build --release && cd -
	./run_qemu.sh --cui

.PHONY: test
test:
	cd kernel && cargo test --release -- --serial
