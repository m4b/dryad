PREFIX=rust
LIB=${PREFIX}/lib
SONAME=dryad.so.1
RUSTLIB=${PREFIX}/lib/rustlib/x86_64-unknown-linux-musl/lib
RUSTHASH=18402db3
#RUSTHASH=$(shell ls $$RUSTLIB/ | grep libstd | grep -oe "-[[:alnum:]]*" | grep -oe "[[:alnum:]]*") # yup you can make fun of me it's cool
RUSTHASH=$(shell ls rust/lib/rustlib/x86_64-unknown-linux-musl/lib/ | grep libstd | grep -oe "-[[:alnum:]]*" | grep -oe "[[:alnum:]]*" | tr -d '[[:space:]]') # yup you can make fun of me it's cool
RUSTHASH:=$(strip ${RUSTHASH})

CARGO=${PREFIX}/bin/cargo
RUSTC=${PREFIX}/bin/rustc

SRC=$(wildcard src/*)

dryad.so.1 : start.o dryad.o
	@echo -e "\E[0;4;33mlinking:\E[0m \E[0;32m${SONAME}\E[0m"
	ld -pie --gc-sections -I/tmp/${SONAME} -L${LIB} -soname ${SONAME} -Bsymbolic -nostdlib -e _start -o ${SONAME} start.o dryad.o "${RUSTLIB}/libstd-${RUSTHASH}.rlib" "${RUSTLIB}/libcore-${RUSTHASH}.rlib" "${RUSTLIB}/librand-${RUSTHASH}.rlib" "${RUSTLIB}/liballoc-${RUSTHASH}.rlib" "${RUSTLIB}/libcollections-${RUSTHASH}.rlib" "${RUSTLIB}/librustc_unicode-${RUSTHASH}.rlib" "${RUSTLIB}/liballoc_system-${RUSTHASH}.rlib" "${RUSTLIB}/libcompiler-rt.a" "${RUSTLIB}/liblibc-${RUSTHASH}.rlib"
	cp ${SONAME} /tmp

start.o : src/arch/x86/asm.s
	@echo -e "\E[0;4;33mcompiling:\E[0m \E[1;30mstart\E[0m"
	gcc -fPIC -c src/arch/x86/asm.s -o start.o

dryad.o : ${SRC}
	@echo -e "\E[0;4;33mcompiling:\E[0m \E[1;32mdryad\E[0m"
	${PREFIX}/bin/rustc --target=x86_64-unknown-linux-musl -g --emit obj -o dryad.o src/lib.rs

clean : start.o dryad.o
	rm *.o ${SONAME}
