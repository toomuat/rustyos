
.PHONY: loader
loader:
	cd loader && mold -run cargo build --release

.PHONY: kernel
kernel:
	cd kernel && mold -run cargo build --release

run: loader kernel
	./run_qemu.sh --cui

all:
	cd loader && mold -run cargo build --release && cd -
	cd kernel && mold -run cargo build --release && cd -
	./run_qemu.sh --cui

.PHONY: test
test:
	cd kernel && mold -run cargo test --release -- --serial
