#!/bin/bash
gcc -c -o test/test.o test/test.c && ld -I /tmp/dryad -lc test/test.o -o test/test && test/test
