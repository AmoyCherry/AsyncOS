set confirm off
set pagination off
set arch riscv:rv64
file D:/async_os/os/target/riscv64gc-unknown-none-elf/release/os
set disassemble-next-line auto
set logging on 3.log
b rust_main
target remote 127.0.0.1:1234
