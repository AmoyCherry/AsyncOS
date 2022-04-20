
#     .align 3
#     .section .data
#     .global _num_app
# _num_app:
#     .quad 1
#     .quad app_0_start


#     .global _app_names
# _app_names:
#     .string "v1"

#     .section .data
#     .global app_0_start
#     .global app_0_end
#     .align 3
# app_0_start:
#     .incbin "../v1/target/riscv64gc-unknown-none-elf/debug/v1"
# app_0_end:
