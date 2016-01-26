#!/bin/bash

PREFIX=musldist
LIB=$PREFIX/lib
SONAME=dryad.so.1
RUSTLIB=$PREFIX/lib/rustlib/x86_64-unknown-linux-musl/lib
RUSTHASH=db5a760f
DEPS_STD=

export LD_LIBRARY_PATH=$PREFIX/lib:$LD_LIBRARY_PATH

clang -c -o start.o src/arch/x86/asm.s

$PREFIX/bin/rustc --target=x86_64-unknown-linux-musl src/main.rs -g -O --emit obj -o dryad.o

# there are still missing symbols:
#ELF X86_64 DYN @ 0x18b0
#Imports (3)
#          22c0b8 __cxa_thread_atexit_impl (0) ~> Unresolved
#          22c0d0 __gcc_personality_v0 (0) ~> Unresolved
#          22c0e8 __dso_handle (0) ~> Unknown
#
#~/projects/rust/dryad $ rdr -m -f __cxa_thread_atexit_impl
#searching /usr/lib/ for __cxa_thread_atexit_impl:
#           363e0 __cxa_thread_atexit_impl (182) -> /usr/lib/libc-2.22.so [libc.so.6]
#~/projects/rust/dryad $ rdr -m -f __gcc_personality_v0
#searching /usr/lib/ for __gcc_personality_v0:
#           122e0 __gcc_personality_v0 (600) -> /usr/lib/libgcc_s.so.1 [libgcc_s.so.1]

ld --gc-sections -I/tmp/$SONAME -lc -L$LIB -soname $SONAME -pie -static -Bsymbolic -nostdlib -shared -e _start -o $SONAME start.o dryad.o "$RUSTLIB/libstd-$RUSTHASH.rlib" "$RUSTLIB/libcore-$RUSTHASH.rlib" "$RUSTLIB/librand-$RUSTHASH.rlib" "$RUSTLIB/liballoc-$RUSTHASH.rlib" "$RUSTLIB/libcollections-$RUSTHASH.rlib" "$RUSTLIB/librustc_unicode-$RUSTHASH.rlib" "$RUSTLIB/liballoc_system-$RUSTHASH.rlib" $LIB/libresolv.a $LIB/libunwind.a $LIB/libm.a $LIB/libc.a

cp $SONAME /tmp/
