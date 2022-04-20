import os


# os.system("cd user && cargo clean")
os.system("cd user && cargo build --release")
os.system('cd basic_rt && cargo build')
os.system('cd basic_rt && rust-objcopy --binary-architecture=riscv64 target/riscv64gc-unknown-none-elf/debug/basic_rt \
            --strip-all -O binary target/riscv64gc-unknown-none-elf/debug/basic_rt.bin')
os.system("cd easy-fs-fuse && cargo run --release -- -s ../user/src/bin/ -t ../user/target/riscv64gc-unknown-none-elf/release/")
os.system("cd os && cargo build --release")
os.system("cd os && cargo build")

# # debug
# os.system("qemu-system-riscv64 \
# -machine virt \
# -nographic \
# -smp cpus=4 \
# -bios bootloader/rustsbi-qemu.bin \
# -device loader,file=os/target/riscv64gc-unknown-none-elf/debug/os,addr=0x80200000 \
# -device loader,file=basic_rt/target/riscv64gc-unknown-none-elf/debug/basic_rt.bin,addr=0x87000000 \
# -drive file=user/target/riscv64gc-unknown-none-elf/release/fs.img,if=none,format=raw,id=x0 \
# -device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0")


# release
os.system("qemu-system-riscv64 \
-machine virt \
-nographic \
-smp cpus=4 \
-bios bootloader/rustsbi-qemu.bin \
-device loader,file=os/target/riscv64gc-unknown-none-elf/release/os,addr=0x80200000 \
-device loader,file=basic_rt/target/riscv64gc-unknown-none-elf/debug/basic_rt.bin,addr=0x87000000 \
-drive file=user/target/riscv64gc-unknown-none-elf/release/fs.img,if=none,format=raw,id=x0 \
-device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0")
