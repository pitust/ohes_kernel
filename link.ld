ENTRY(_start)
 
/* Tell where the various sections of the object files will be put in the final
   kernel image. */
SECTIONS
{
	/* Begin putting sections at 1 MiB, a conventional place for kernels to be
	   loaded at by the bootloader. */
	. = 1M;
 
	/* First put the multiboot header, as it is required to be put very early
	   early in the image or the bootloader won't recognize the file format.
	   Next we'll put the .text section. */
	.text : ALIGN(4K)
	{
		*(.multiboot_header)
		*(.text)
		*(.text.*)
	}
 
	/* Read-only data. */
	.rodata : ALIGN(4K)
	{
		*(.rodata)
		*(.rodata.*)
	}
 
	/* Read-write data (initialized) */
	.data : ALIGN(4K)
	{
		*(.data)
		*(.data.*)
	}
 
	/* Read-write data (uninitialized) and stack */
	.bss : ALIGN(4K)
	{
		*(.bss)
		*(.bss.*)
	}
	.got : ALIGN(4K)
	{
		*(.got)
		*(.got.*)
	}
	es = .;
	.eh_frame : ALIGN(4K)
	{
		*(.eh_frame)
		*(.eh_frame.*)
	}
	ee = .;
	esz = ee - es;
 
	/* The compiler may produce other sections, by default it will put them in
	   a segment with the same name. Simply add stuff here as needed. */
}