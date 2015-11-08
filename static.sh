#!/bin/bash

PREFIX=musldist
LIB=$PREFIX/lib
SONAME=dryad.so.1
RUSTLIB=$PREFIX/lib/rustlib/x86_64-unknown-linux-musl/lib
DEPS_STD=

#LIBS=

export LD_LIBRARY_PATH=$PREFIX/lib:$LD_LIBRARY_PATH

clang -c -o start.o src/arch/x86/asm.s

$PREFIX/bin/rustc --target=x86_64-unknown-linux-musl src/main.rs -g -O --emit obj -o dryad.o

ld -I/tmp/$SONAME -lc -L$LIB -soname $SONAME -pie -static -Bsymbolic -nostdlib -shared -e _start -o $SONAME start.o dryad.o $RUSTLIB/libstd-71b07a99.rlib $RUSTLIB/libcore-71b07a99.rlib $RUSTLIB/librand-71b07a99.rlib $RUSTLIB/liballoc-71b07a99.rlib $RUSTLIB/libcollections-71b07a99.rlib $RUSTLIB/librustc_unicode-71b07a99.rlib $RUSTLIB/liballoc_system-71b07a99.rlib $LIB/libm.a $LIB/libc.a

cp $SONAME /tmp/

#$PREFIX/lib/rustlib/x86_64-unknown-linux-musl/lib/libcore-71b07a99.rlib
#ld -pie -static -Bsymbolic -nostdlib -shared -e _start -o main start.o main.o $PREFIX/lib/rustlib/x86_64-unknown-linux-musl/lib/libstd-71b07a99.rlib $LIB/libm.a $LIB/libc.a
