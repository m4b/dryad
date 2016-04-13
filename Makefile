#PREFIX=rust
# shouldn't have to hardcode this fragile dir structure, but for some reason rustup wants to name nightly after host
PREFIX=$(HOME)/.multirust/toolchains/nightly-x86_64-unknown-linux-gnu
LIB=$(PREFIX)/lib
RUSTLIB=$(LIB)/rustlib/x86_64-unknown-linux-musl/lib
HASH=$(shell ls $(RUSTLIB) | grep libstd | grep -oe "-[[:alnum:]]*" | grep -oe "[[:alnum:]]*")
RUSTHASH=$(strip $(HASH))

SONAME=dryad.so.1

CARGO=$(shell which cargo)

SRC=$(wildcard src/*)

dryad.so.1 : start.o dryad.o
	@echo -e "\E[0;4;33mlinking:\E[0m \E[0;32m$(SONAME)\E[0m with $(HASH)"
	ld -pie --gc-sections -I/tmp/${SONAME} -L${LIB} -soname ${SONAME} -Bsymbolic -nostdlib -e _start -o ${SONAME} start.o dryad.o "${RUSTLIB}/libstd-${RUSTHASH}.rlib" "${RUSTLIB}/libcore-${RUSTHASH}.rlib" "${RUSTLIB}/librand-${RUSTHASH}.rlib" "${RUSTLIB}/liballoc-${RUSTHASH}.rlib" "${RUSTLIB}/libcollections-${RUSTHASH}.rlib" "${RUSTLIB}/librustc_unicode-${RUSTHASH}.rlib" "${RUSTLIB}/liballoc_system-${RUSTHASH}.rlib" "${RUSTLIB}/libcompiler-rt.a" "${RUSTLIB}/liblibc-${RUSTHASH}.rlib" "target/x86_64-unknown-linux-musl/debug/deps/libcrossbeam-be36913e782f04c9.rlib"
	cp ${SONAME} /tmp

start.o : src/arch/x86/asm.s
	@echo -e "\E[0;4;33mcompiling:\E[0m \E[1;30mstart\E[0m"
	gcc -fPIC -c src/arch/x86/asm.s -o start.o

dryad.o : ${SRC}
	@echo -e "\E[0;4;33mcompiling:\E[0m \E[1;32mdryad\E[0m"
	$(CARGO) rustc --verbose --target=x86_64-unknown-linux-musl --lib -j 4 -- --emit obj -o dryad.o

clean :
	cargo clean
	rm *.o
	rm ${SONAME}
	rm *.d

# todo: add make test target, remove gen_tests.sh
