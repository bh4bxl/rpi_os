PAGE_SIZE = 64K;
PAGE_MASK = PAGE_SIZE - 1;

__rpi_phys_dram_start_addr = 0;

__rpi_phys_binary_load_addr = 0x80000;

ENTRY(__rpi_phys_binary_load_addr)
PHDRS
{
    segment_boot_core_stack PT_LOAD FLAGS(6);
    segment_code PT_LOAD FLAGS(5);
    segment_data PT_LOAD FLAGS(6);
}

SECTIONS
{
    . =  __rpi_phys_dram_start_addr;

    /* Boot Core Stack */
    .boot_core_stack (NOLOAD) :
    {
        . += __rpi_phys_binary_load_addr;
        __boot_core_stack_end_exclusive = .;
    } :segment_boot_core_stack

    ASSERT((. & PAGE_MASK) == 0, "End of boot core stack is not page aligned")

    /* Code + RO Data + Global Offset Table */
    __code_start = .;
    .text :
    {
        KEEP(*(.text._start))
        *(.text._start_arguments)
        *(.text._rust_start)
        *(.text*)
    } :segment_code

    .rodata : ALIGN(8) { *(.rodata*) } :segment_code

    . = ALIGN(PAGE_SIZE);
    __code_end_exclusive = .;

    /* Data + BSS */
    .data : { *(.data*) } :segment_data

    .bss (NOLOAD) : ALIGN(16)
    {
        __bss_start = .;
        *(.bss*);
        . = ALIGN(16);
        __bss_end_exclusive = .;
    } :segment_data

    .got : { *(.got*) }
    ASSERT(SIZEOF(.got) == 0, "Relocation support not expected")

    /DISCARD/ : { *(.comment*) }
}
