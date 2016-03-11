#!/bin/bash
set -e

PREFIX=musldist
LIB=$PREFIX/lib
DRYAD=/tmp/dryad.so.1
TESTDIR=test

echo -e "PT_INTERPRETER for $TESTDIR/ binaries is $DRYAD\nBinaries prefixed with 'ld' use the system dynamic linker, ld-linux-x86-64.so.2"

echo -e "Building regular binary $TESTDIR/test with libm and libc"
gcc -lm -Wl,-I,$DRYAD $TESTDIR/test.c -o $TESTDIR/test
gcc -lm $TESTDIR/test.c -o $TESTDIR/ldtest

echo -e "Building thread local binary $TESTDIR/tlocal with pthreads and libc"
gcc -lpthread -Wl,-I,$DRYAD $TESTDIR/tlocal.c -o $TESTDIR/tlocal
gcc -lpthread $TESTDIR/tlocal.c -o $TESTDIR/ldtlocal

echo -e "Building complicated binary $TESTDIR/snappy linked with libm and snappy (on my system snappy uses libstdc++.so.6, libm.so.6, libc.so.6, and libgcc_s.so.1)"
gcc -lm -lsnappy -Wl,-I,$DRYAD $TESTDIR/snappy.c -o $TESTDIR/snappy
gcc -lm -lsnappy $TESTDIR/snappy.c -o $TESTDIR/ldsnappy

# uncomment this and use $LIB with a `libc.so` to create a musl binary to test with
#echo -e "Building musl linked binary $TESTDIR/musl"
#gcc -nodefaultlibs -nostdlib -Wl,-I$DRYAD -L$LIB -lc $LIB/Scrt1.o $TESTDIR/musl.c -o $TESTDIR/musl
#gcc -nodefaultlibs -nostdlib -L$LIB -lc $LIB/Scrt1.o $TESTDIR/musl.c -o $TESTDIR/ldmusl
#
#export LD_LIBRARY_PATH=/home/m4b/src/musl-1.1.14/lib
#echo -e "to run, do: \$LD_LIBRARY_PATH=$LD_LIBRARY_PATH test/musl"

