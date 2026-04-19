ENTRY(_start)

MEMORY {
    FLASH : ORIGIN = 0x10040000, LENGTH = 1792K
    RAM : ORIGIN = 0x20020000, LENGTH = 128K
}

SECTIONS
{
    . = 0x10040000;

    .text : {
        KEEP(*(.text._start))
        *(.text*)
    } > FLASH

    .rodata : { *(.rodata*) } > FLASH

    .data : {
        _data_start = .;
        *(.data*)
        _data_end = .;
    } > RAM AT > FLASH

    _data_load = LOADADDR(.data);

    .bss : {
        _bss_start = .;
        *(.bss*)
        *(COMMON)
        _bss_end = .;
    } > RAM

    /DISCARD/ : {
        *(.comment*)
        *(.note*)
        *(.ARM.exidx*)
        *(.ARM.extab*)
    }
}
