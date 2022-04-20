import os

os.system("cd user && cargo build --release")
os.system("cd basic_rt && python build.py")
os.system("cd easy-fs-fuse && cargo run --release -- -s ../user/src/bin/ -t ../user/target/riscv64gc-unknown-none-elf/release/")
os.system("cd os && cargo build --release")
os.system("cd os && cargo build")
