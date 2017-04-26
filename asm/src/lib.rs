#![feature(asm)]
#![feature(naked_functions)]

#[no_mangle]
// pub extern fn _dryad_fini() {
// 	leaq (%rip), %rax
//         retq

#[no_mangle]
#[naked]
pub extern fn _start() {
    // TODO: do stupid i386 thunk asm nonsense
    #[cfg(target_arch = "x86")]
    unsafe {
        asm!("
        mov %esp, %edi
        and $$~15, %esp
        call dryad_init
        mov $0, %edx
        jmp *%eax
        "
        );
    }
    #[cfg(target_arch = "x86_64")]
    unsafe {
        asm!("
        mov %rsp, %rdi
        andq $$~15, %rsp
        callq dryad_init
        movq $$0, %rdx
        jmpq *%rax
        "
        );
    }
    #[cfg(target_arch = "arm")]
    unsafe {
        asm!("
	mov r0, sp
	bl dryad_init
	mov pc, r0
        bx pc
        "
        );
    }
    #[cfg(target_arch = "arm64")]
    unsafe {
        asm!("
    	mov x0, sp
	bl dryad_init
	br x0
        ");
    }
}

#[no_mangle]
pub extern fn _dryad_resolve_symbol () {
    #[cfg(target_arch = "x86")]
    unsafe {
        asm!("
        ");
    }
    #[cfg(target_arch = "x86_64")]
    unsafe {
        asm!("
	sub    $$0x180,%rsp
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
	bndmov %bnd0,0x100(%rsp)
	bndmov %bnd1,0x110(%rsp)
	bndmov %bnd2,0x120(%rsp)
	bndmov %bnd3,0x130(%rsp)
	mov    0x10(%rbx),%rsi
	mov    0x8(%rbx),%rdi
	callq dryad_resolve_symbol
	mov    %rax,%r11
	bndmov 0x130(%rsp),%bnd3
	bndmov 0x120(%rsp),%bnd2
	bndmov 0x110(%rsp),%bnd1
	bndmov 0x100(%rsp),%bnd0
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
	add    $$0x18,%rsp
	jmpq *%r11
        "
        );
    }
    #[cfg(target_arch = "arm")]
    unsafe {
        asm!("
	and    sp, #-15
	bl dryad_resolve_symbol
        ");
    }
    #[cfg(target_arch = "arm64")]
    unsafe {
        asm!("
	bl dryad_resolve_symbol
        ");
    }
}

