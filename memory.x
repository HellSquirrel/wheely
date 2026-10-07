MEMORY {
    /* Pico 2 W has 4 MiB of QSPI flash. */
    FLASH : ORIGIN = 0x10000000, LENGTH = 4096K
    /* SRAM0-SRAM7, striped. */
    RAM : ORIGIN = 0x20000000, LENGTH = 512K
    /* SRAM8/9, direct-mapped; handy later for per-core stacks. */
    SRAM8 : ORIGIN = 0x20080000, LENGTH = 4K
    SRAM9 : ORIGIN = 0x20081000, LENGTH = 4K
}

SECTIONS {
    /* Boot ROM image definition. Must stay in the first 4K of flash. */
    .start_block : ALIGN(4)
    {
        __start_block_addr = .;
        KEEP(*(.start_block));
        KEEP(*(.boot_info));
    } > FLASH
} INSERT AFTER .vector_table;

/* Move .text to start after the boot info. */
_stext = ALIGN(ADDR(.start_block) + SIZEOF(.start_block), 8);

SECTIONS {
    /* picotool 'Binary Info' entries. */
    .bi_entries : ALIGN(4)
    {
        __bi_entries_start = .;
        KEEP(*(.bi_entries));
        . = ALIGN(4);
        __bi_entries_end = .;
    } > FLASH
} INSERT AFTER .text;

SECTIONS {
    /* Boot ROM end block; goes after everything so it can hold a signature. */
    .end_block : ALIGN(4)
    {
        __end_block_addr = .;
        KEEP(*(.end_block));
    } > FLASH
} INSERT AFTER .uninit;

PROVIDE(start_to_end = __end_block_addr - __start_block_addr);
PROVIDE(end_to_start = __start_block_addr - __end_block_addr);
