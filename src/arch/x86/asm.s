        .text
        .globl _start
        .type _start, @function
_start:
        mov %rsp, %rdi
        callq _dryad_init
        jmpq *%rax
        retq

        .text
        .globl _dryad_resolve_symbol
        .type _dryad_resolve_symbol, @function
_dryad_resolve_symbol:
        mov %rsp, %rdi
	andq $-16, %rsp
        callq dryad_resolve_symbol
        jmpq *%rax

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
