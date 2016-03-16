#!/bin/bash

set -e

PREFIX=musldist #can almost use new rust deploy system at /usr/local but getting relocation errors w.r.t. liblibc wrapper :/
LIB=$PREFIX/lib
SONAME=dryad.so.1
RUSTLIB=$PREFIX/lib/rustlib/x86_64-unknown-linux-musl/lib
RUSTHASH=$(ls $RUSTLIB/ | grep libstd | grep -oe "-[[:alnum:]]*" | grep -oe "[[:alnum:]]*") # yup you can make fun of me it's cool
echo -e "using rust hash $RUSTHASH from $PREFIX"

export LD_LIBRARY_PATH=$PREFIX/lib #appending empty :$LD_LIBRARY_PATH causes segfault since it grabs libc.so.6 sitting in dryad dir, which has some kind of binary incompat over latest version because why not

echo -e "compiling asm..."
gcc -fPIC -c -o start.o src/arch/x86/asm.s

echo -e "compiling dryad..."
$PREFIX/bin/rustc --target=x86_64-unknown-linux-musl src/lib.rs -g --emit obj -o dryad.o

# there are still missing symbols:
#ELF X86_64 DYN @ 0x18b0
#Imports (3)
#          22c0b8 __cxa_thread_atexit_impl (0) ~> Unresolved
#
# it is present in glibc but not musl :/
#
#~/projects/rust/dryad $ rdr -m -f __cxa_thread_atexit_impl
#searching /usr/lib/ for __cxa_thread_atexit_impl:
#           363e0 __cxa_thread_atexit_impl (182) -> /usr/lib/libc-2.22.so [libc.so.6]

echo -e "linking..."
# using -shared results in DPTMOD64 reloc, and because tls not properly init'd for __tls_get_address (only for local exec) inside of dryad, everything breaks
ld -pie --gc-sections -I/tmp/$SONAME -L$LIB -soname $SONAME -Bsymbolic -nostdlib -e _start -o $SONAME start.o dryad.o "$RUSTLIB/libstd-$RUSTHASH.rlib" "$RUSTLIB/libcore-$RUSTHASH.rlib" "$RUSTLIB/librand-$RUSTHASH.rlib" "$RUSTLIB/liballoc-$RUSTHASH.rlib" "$RUSTLIB/libcollections-$RUSTHASH.rlib" "$RUSTLIB/librustc_unicode-$RUSTHASH.rlib" "$RUSTLIB/liballoc_system-$RUSTHASH.rlib" "$RUSTLIB/libcompiler-rt.a" $LIB/libc.a

# use this when fixed: https://internals.rust-lang.org/t/static-binary-support-in-rust/2011/55
#"$RUSTLIB/liblibc-$RUSTHASH.rlib"

cp $SONAME /tmp/
