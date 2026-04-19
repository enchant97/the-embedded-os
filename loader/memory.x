MEMORY {
    BOOT2 : ORIGIN = 0x10000000, LENGTH = 0x100
    /* define kernel memory areas */
    FLASH : ORIGIN = 0x10000100, LENGTH = 256K - 0x100
    RAM   : ORIGIN = 0x20000000, LENGTH = 128K
    /* define app section memory areas */
    APP_FLASH : ORIGIN = 0x10040000, LENGTH = 1792K
    APP_RAM : ORIGIN = 0x20020000, LENGTH = 128K
    /* shared, for future use with syscalls */
    IPC_RAM : ORIGIN = 0x20040000, LENGTH = 4K
}

SECTIONS {
    .app_flash_slot : {
        KEEP(*(.app_flash_slot))
    } > APP_FLASH
}
