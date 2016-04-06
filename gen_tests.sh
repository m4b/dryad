#!/bin/bash
set -e

PREFIX=musldist
LIB=$PREFIX/lib
DRYAD=/tmp/dryad.so.1
TESTDIR=test

echo -e "PT_INTERPRETER for $TESTDIR/ binaries is $DRYAD\nBinaries prefixed with 'ld' use the system dynamic linker, ld-linux-x86-64.so.2"

echo -e "Building regular binary $TESTDIR/test with libm and libc"
gcc -Wl,-I,$DRYAD $TESTDIR/test.c -o $TESTDIR/test -lm
gcc $TESTDIR/test.c -o $TESTDIR/ldtest -lm

echo -e "Building thread local binary $TESTDIR/tlocal with pthreads and libc"
gcc -Wl,-I,$DRYAD $TESTDIR/tlocal.c -o $TESTDIR/tlocal -lpthread
gcc $TESTDIR/tlocal.c -o $TESTDIR/ldtlocal -lpthread

echo -e "Building complicated binary $TESTDIR/snappy linked with libm and snappy (on my system snappy uses libstdc++.so.6, libm.so.6, libc.so.6, and libgcc_s.so.1)"
gcc -Wl,-I,$DRYAD $TESTDIR/snappy.c -o $TESTDIR/snappy -lm -lsnappy
gcc $TESTDIR/snappy.c -o $TESTDIR/ldsnappy -lm -lsnappy

echo -e "Building a binary $TESTDIR/float with libm that performs no printing"
gcc -Wl,-I,$DRYAD $TESTDIR/float.c -o $TESTDIR/float -lm
gcc $TESTDIR/float.c -o $TESTDIR/ldfloat -lm

echo -e "Building a binary $TESTDIR/getaux which calls getauxval(AT_ENTRY), and will segfault if the proper global struct isn't setup depending on which libc we are"
gcc -Wl,-I,$DRYAD $TESTDIR/getaux.c -o $TESTDIR/getaux
gcc $TESTDIR/getaux.c -o $TESTDIR/ldgetaux

# uncomment this and use $LIB with a `libc.so` to create a musl binary to test with
#echo -e "Building musl linked binary $TESTDIR/musl"
#gcc -nodefaultlibs -nostdlib -Wl,-I$DRYAD -L$LIB -lc $LIB/Scrt1.o $TESTDIR/musl.c -o $TESTDIR/musl
#gcc -nodefaultlibs -nostdlib -L$LIB -lc $LIB/Scrt1.o $TESTDIR/musl.c -o $TESTDIR/ldmusl
#
#export LD_LIBRARY_PATH=/home/m4b/src/musl-1.1.14/lib
#echo -e "to run, do: \$LD_LIBRARY_PATH=$LD_LIBRARY_PATH test/musl"

