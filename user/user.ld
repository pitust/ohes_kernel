SECTIONS {
    . = 0xFFFF800000000000;
    .text : { 
        *(.text)
    }
    . = ALIGN(4096);
    .data : { 
        *(.data)
    }
}