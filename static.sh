#!/bin/bash

set -e

PREFIX=musldist #change me if you installed rust somewhere different
LIB=$PREFIX/lib
SONAME=dryad.so.1
RUSTLIB=$PREFIX/lib/rustlib/x86_64-unknown-linux-musl/lib
RUSTHASH=$(ls $RUSTLIB/ | grep libstd | grep -oe "-[[:alnum:]]*" | grep -oe "[[:alnum:]]*") # yup you can make fun of me it's cool
echo -e "using rust hash $RUSTHASH"

export LD_LIBRARY_PATH=$PREFIX/lib #appending :$LD_LIBRARY_PATH causes segfault since it grabs libc.so.6 sitting in dryad dir, which has some kind of binary incompat over latest version because why not

echo -e "compiling asm..."
gcc -fPIC -c -o start.o src/arch/x86/asm.s

echo -e "compiling dryad..."
$PREFIX/bin/rustc --target=x86_64-unknown-linux-musl src/lib.rs -g --emit obj -o dryad.o

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

#add -E to force all symbols to be exported, good for testing
echo -e "linking..."
# using -shared results in DPTMOD64 reloc, and because tls not properly init'd for __tls_get_address (only for local exec) inside of dryad, everything breaks
ld --gc-sections -I/tmp/$SONAME -lc -L$LIB -soname $SONAME -pie -Bsymbolic -nostdlib -e _start -o $SONAME start.o dryad.o "$RUSTLIB/libstd-$RUSTHASH.rlib" "$RUSTLIB/libcore-$RUSTHASH.rlib" "$RUSTLIB/librand-$RUSTHASH.rlib" "$RUSTLIB/liballoc-$RUSTHASH.rlib" "$RUSTLIB/libcollections-$RUSTHASH.rlib" "$RUSTLIB/librustc_unicode-$RUSTHASH.rlib" "$RUSTLIB/liballoc_system-$RUSTHASH.rlib" "$RUSTLIB/libcompiler-rt.a" $LIB/libresolv.a $LIB/libunwind.a $LIB/libm.a $LIB/libc.a

cp $SONAME /tmp/
