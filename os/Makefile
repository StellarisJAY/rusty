rust-sbi-run:build
	@qemu-system-riscv64 \
	-machine virt \
	-nographic \
	-bios ../bootloader/rustsbi-qemu.bin \
	-device loader,file=target/riscv64gc-unknown-none-elf/release/os.bin,addr=0x80200000
run:build
	@qemu-system-riscv64 \
	--machine virt \
	--nographic \
	--bios default \
	--kernel target/riscv64gc-unknown-none-elf/release/os.bin
rust-sbi-debug:build
	@qemu-system-riscv64 \
	-machine virt \
	-nographic \
	-bios ../bootloader/rustsbi-qemu.bin \
	-device loader,file=target/riscv64gc-unknown-none-elf/release/os.bin,addr=0x80200000 -s -S
build: 
	@cargo build --release
	@rust-objcopy --strip-all target/riscv64gc-unknown-none-elf/release/os \
	-O binary target/riscv64gc-unknown-none-elf/release/os.bin
debug:build
	@qemu-system-riscv64 \
        -machine virt \
        -nographic \
        -bios default \
	-kernel target/riscv64gc-unknown-none-elf/release/os.bin -s -S
gdb:
	riscv64-unknown-elf-gdb -ex 'file target/riscv64gc-unknown-none-elf/release/os' -ex 'set arch riscv:rv64' -ex 'target remote localhost:1234'
