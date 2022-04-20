import os

os.system('cargo clean')
os.system('cargo build')
os.system('cargo build --release')
os.system('rust-objcopy --binary-architecture=riscv64 target/riscv64gc-unknown-none-elf/debug/basic_rt \
            --strip-all -O binary target/riscv64gc-unknown-none-elf/debug/basic_rt.bin')
os.system('rust-objcopy --binary-architecture=riscv64 target/riscv64gc-unknown-none-elf/release/basic_rt \
            --strip-all -O binary target/riscv64gc-unknown-none-elf/release/basic_rt.bin')
# os.system('qemu-system-riscv64 -machine virt -nographic -no-reboot -bios default -device loader,file=target/riscv64gc-unknown-none-elf/debug/tiny_os_a,addr=0x80200000')
