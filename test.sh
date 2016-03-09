#!/bin/bash

echo -e "Building musl linked binary"
gcc -nodefaultlibs -nostdlib -Wl,-I/tmp/dryad.so.1 -L/home/m4b/src/musl-1.1.14/lib -lc /home/m4b/src/musl-1.1.14/lib/crt1.o test/musl.c -o musl

export LD_LIBRARY_PATH=/home/m4b/src/musl-1.1.14/lib
echo -e "running musl binary with $LD_LIBRARY_PATH"
./musl

