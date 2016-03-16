	.text
        .globl _dryad_fini
        .type _dryad_fini, @function
_dryad_fini:
	leaq (%rip), %rax
        retq

	// TODO: fix in lib.rs: _start needs to get a stack and argc that looks like it was executed directly, i believe this might be the final cause of the segfault in dynamic linking, because certain arguments in the stack are too high?
	.text
        .globl _start
        .type _start, @function
_start:
//	xor %ebp, %ebp
	mov %rsp, %rdi
//	mov %rsp, %rbp ; we shouldn't need to save rbp
	andq $~15, %rsp
        callq _dryad_init
	/*
	mov %rax, %rcx
	callq _dryad_fini
	movq %rax, %rdx
	movq %rcx, %rax

	// crashes here: 
	__new_exitfn

	*/
	movq $0, %rdx
        jmpq *%rax
        retq

        .text
        .globl _dryad_resolve_symbol
        .type _dryad_resolve_symbol, @function
_dryad_resolve_symbol:
	push   %rbx
	mov    %rsp,%rbx
	and    $0xffffffffffffffe0,%rsp
	sub    $0x180,%rsp
	mov    %rax,0x140(%rsp)
	mov    %rcx,0x148(%rsp)
	mov    %rdx,0x150(%rsp)
	mov    %rsi,0x158(%rsp)
	mov    %rdi,0x160(%rsp)
	mov    %r8,0x168(%rsp)
	mov    %r9,0x170(%rsp)
	vmovdqa %ymm0,(%rsp)
	vmovdqa %ymm1,0x20(%rsp)
	vmovdqa %ymm2,0x40(%rsp)
	vmovdqa %ymm3,0x60(%rsp)
	vmovdqa %ymm4,0x80(%rsp)
	vmovdqa %ymm5,0xa0(%rsp)
	vmovdqa %ymm6,0xc0(%rsp)
	vmovdqa %ymm7,0xe0(%rsp)
//	bndmov %bnd0,0x100(%rsp)
//	bndmov %bnd1,0x110(%rsp)
//	bndmov %bnd2,0x120(%rsp)
//	bndmov %bnd3,0x130(%rsp)
	mov    0x10(%rbx),%rsi
	mov    0x8(%rbx),%rdi
	callq  dryad_resolve_symbol
	mov    %rax,%r11
//	bndmov 0x130(%rsp),%bnd3
//	bndmov 0x120(%rsp),%bnd2
//	bndmov 0x110(%rsp),%bnd1
//	bndmov 0x100(%rsp),%bnd0
	mov    0x170(%rsp),%r9
	mov    0x168(%rsp),%r8
	mov    0x160(%rsp),%rdi
	mov    0x158(%rsp),%rsi
	mov    0x150(%rsp),%rdx
	mov    0x148(%rsp),%rcx
	mov    0x140(%rsp),%rax
	vmovdqa (%rsp),%ymm0
	vmovdqa 0x20(%rsp),%ymm1
	vmovdqa 0x40(%rsp),%ymm2
	vmovdqa 0x60(%rsp),%ymm3
	vmovdqa 0x80(%rsp),%ymm4
	vmovdqa 0xa0(%rsp),%ymm5
	vmovdqa 0xc0(%rsp),%ymm6
	vmovdqa 0xe0(%rsp),%ymm7
	mov    %rbx,%rsp
	mov    (%rsp),%rbx
	add    $0x18,%rsp
	jmpq *%r11
	nopl   0x0(%rax)
	
	.text
        .globl _print
        .type _print, @function
_print:
        push %rbp
        mov %rsp, %rbp
        mov %rsi, %rdx
        mov %rdi, %rsi
        mov $1, %rax
        mov $1, %rdi
        syscall
        pop %rbp
        retq
