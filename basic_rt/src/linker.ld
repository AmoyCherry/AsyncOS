OUTPUT_ARCH(riscv)
ENTRY(_start)

BASE_ADDRESS = 0x87000000;

SECTIONS
{
    . = BASE_ADDRESS;
    start = .;
    .text : {
        KEEP(*(.text*))
        stext = .;
        *(.text.entry)
        *(.text .text.*)
        . = ALIGN(4K);
        etext = .;
    }
    .rodata : {
        srodata = .;
        *(.rodata .rodata.*)
        . = ALIGN(4K);
        erodata = .;
    }
    .data : {
        sdata = .;
        *(.data .data.*)
        . = ALIGN(4K);
        edata = .;
    }

    .stack : {
        *(.bss.stack)
        . = ALIGN(4K);
    }

    .bss : {
        sbss = .;
        *(.bss .bss.*)
        . = ALIGN(4K);
        ebss = .;
    }
    PROVIDE(end = .);
}
