run:
	@qemu-system-riscv64 \
	-machine virt \
	-nographic \
	-bios ../bootloader/rustsbi-qemu.bin \
	-device loader,file=target/riscv64gc-unknown-none-elf/release/00hello_world.bin,addr=0x80400000
build:
	@cargo build --release
debug:build
	@qemu-system-riscv64 \
        -machine virt \
        -nographic \
        -bios ../bootloader/rustsbi-qemu.bin \
        -device loader,file=target/riscv64gc-unknown-none-elf/release/user.bin,addr=0x80200000 -s -S

env:
	@rustup install nightly
	@rustup default nightly
	@rustup target add riscv64gc-unknown-none-elf
	@cargo install cargo-binutils
	@rustup component add llvm-tools-preview
	@rustup component add rust-src
run00:build
	@qemu-riscv64 target/riscv64gc-unknown-none-elf/release
