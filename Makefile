PREFIX=$(HOME)/.multirust/toolchains/nightly-x86_64-unknown-linux-gnu
LIB=$(PREFIX)/lib
RUSTLIB=$(LIB)/rustlib/x86_64-unknown-linux-musl/lib
HASH=$(shell ls $(RUSTLIB) | grep libstd | grep -oe "-[[:alnum:]]*" | grep -oe "[[:alnum:]]*")
RUSTHASH=$(strip $(HASH))

SRC=$(wildcard src/*)
OUT_DIR=target/x86_64-unknown-linux-musl/debug
CARGO=$(shell which cargo)

# adds 300KB, 300 more runtime relocations, and segfaults the binary
#RUSTLIBS=$(wildcard $(RUSTLIB)/*.rlib $(RUSTLIB)/*.a)
# this needs better handling, a la discussion with ubsan and Mutabah
CARGO_DEPS=$(wildcard target/x86_64-unknown-linux-musl/debug/deps/*.rlib)
# this is a hack because of extra 300KB and segfaulting
RUSTLIBS := "${RUSTLIB}/libstd-${RUSTHASH}.rlib" "${RUSTLIB}/libcore-${RUSTHASH}.rlib" "${RUSTLIB}/librand-${RUSTHASH}.rlib" "${RUSTLIB}/liballoc-${RUSTHASH}.rlib" "${RUSTLIB}/libcollections-${RUSTHASH}.rlib" "${RUSTLIB}/librustc_unicode-${RUSTHASH}.rlib" "${RUSTLIB}/liballoc_system-${RUSTHASH}.rlib" "${RUSTLIB}/libcompiler-rt.a" "${RUSTLIB}/liblibc-${RUSTHASH}.rlib"

SONAME=dryad.so.1

LINK_ARGS := -pie -I/tmp/${SONAME} -soname ${SONAME} --gc-sections -L${LIB} -Bsymbolic -nostdlib -e _start

dryad.so.1 : $(OUT_DIR)/libdryad.rlib
	@printf "\33[0;4;33mlinking:\33[0m \33[0;32m$(SONAME)\33[0m with $(HASH)\n"
	ld ${LINK_ARGS} -o ${SONAME} ${OUT_DIR}/libdryad.rlib ${RUSTLIBS} ${CARGO_DEPS}
	cp ${SONAME} /tmp

$(OUT_DIR)/libdryad.rlib : ${SRC}
	@printf "\33[0;4;33mcompiling:\33[0m \33[1;32mdryad\33[0m\n"
	$(CARGO) rustc --verbose --target=x86_64-unknown-linux-musl --lib -j 4

#almost... but cargo/rustc refuses to compile dylibs with a musl target
#link-args="-Wl,-pie,-I/tmp/${SONAME},-soname ${SONAME}, --gc-sections, -L${LIB}, -Bsymbolic, -nostdlib, -e _start, -o ${SONAME}, start.o, dryad.o, ${RUSTLIB}/libstd-${RUSTHASH}.rlib, ${RUSTLIB}/libcore-${RUSTHASH}.rlib, ${RUSTLIB}/librand-${RUSTHASH}.rlib, ${RUSTLIB}/liballoc-${RUSTHASH}.rlib, ${RUSTLIB}/libcollections-${RUSTHASH}.rlib, ${RUSTLIB}/librustc_unicode-${RUSTHASH}.rlib, ${RUSTLIB}/liballoc_system-${RUSTHASH}.rlib, ${RUSTLIB}/libcompiler-rt.a, ${RUSTLIB}/liblibc-${RUSTHASH}.rlib, ${CARGO_DEPS}"

clean :
	cargo clean
	rm ${SONAME}

run : dryad.so.1
	./dryad.so.1

# todo: add make test target, remove gen_tests.sh

