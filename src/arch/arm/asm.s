	.text
        .globl _start
//        .type _start, @function
_start:
	mov r0, sp
	bl dryad_init
	mov pc, r0

	.text
        .globl _dryad_resolve_symbol
//        .type _dryad_resolve_symbol, @function
_dryad_resolve_symbol:
//	push   %rbx
//	mov    %rsp,%rbx
	and    sp, #-15
	bl dryad_resolve_symbol
