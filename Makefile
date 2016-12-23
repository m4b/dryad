SONAME=dryad.so.1
###################### Config
TRIPLE=aarch64-linux-android
CC=/opt/android-ndk/toolchains/aarch64-linux-android-4.9/prebuilt/linux-x86_64/bin/aarch64-linux-android-gcc
LD=/opt/android-ndk/toolchains/aarch64-linux-android-4.9/prebuilt/linux-x86_64/bin/aarch64-linux-android-ld
AR=/opt/android-ndk/toolchains/aarch64-linux-android-4.9/prebuilt/linux-x86_64/bin/aarch64-linux-android-ar
CARGO=$(shell which cargo)
# set this for android, et., al
USRLIB=/opt/android-ndk/platforms/android-24/arch-arm64/usr/lib/
PT_INTERP=/data/data/com.termux/files/home/${SONAME}
#############################
TRIPLE=arm-unknown-linux-musleabi
CC=/opt/android-ndk/toolchains/arm-linux-androideabi-4.9/prebuilt/linux-x86_64/bin/arm-linux-androideabi-gcc
LD=/opt/android-ndk/toolchains/arm-linux-androideabi-4.9/prebuilt/linux-x86_64/bin/arm-linux-androideabi-ld
AR=/opt/android-ndk/toolchains/arm-linux-androideabi-4.9/prebuilt/linux-x86_64/bin/arm-linux-androideabi-ar
CARGO=$(shell which cargo)
# set this for android, et., al
USRLIB=/opt/android-ndk/platforms/android-24/arch-arm/usr/lib/
PT_INTERP=/data/data/com.termux/files/home/${SONAME}
#############################
TRIPLE=x86_64-unknown-linux-musl
CC=gcc
LD=ld
AR=ar
CARGO=$(shell which cargo)
USRLIB=/usr/lib
PT_INTERP=/tmp/${SONAME}

CCOPT=-g -O0
PREFIX=$(HOME)/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu
LIB=$(PREFIX)/lib
RUSTLIB=$(LIB)/rustlib/$(TRIPLE)/lib
HASH=$(shell ls $(RUSTLIB) | grep "libstd.*.rlib" | grep -oe "-[[:alnum:]]*" | grep -oe "[[:alnum:]]*")
RUSTHASH=$(strip $(HASH))

# uncomment to remove color from dryad :(
#COLOR=--features "no_color"
ETC=etc
SRC=$(wildcard src/*)
OUT_DIR=target/$(TRIPLE)/debug

# adds 300KB, 300 more runtime relocations, and segfaults the binary
RUSTLIBS=$(wildcard $(RUSTLIB)/*.rlib $(RUSTLIB)/*.a)
# this needs better handling, a la discussion with ubsan and Mutabah
CARGO_DEPS=$(wildcard target/$(TRIPLE)/debug/deps/*.rlib)
# this is a hack because of extra 300KB and segfaulting
# RUSTLIBS := $(addprefix $(RUSTLIB),\
# 	/libstd-$(RUSTHASH).rlib\
# 	 /libcore-$(RUSTHASH).rlib\
# 	 /librand-$(RUSTHASH).rlib\
# 	 /liballoc-$(RUSTHASH).rlib\
# 	 /libcollections-$(RUSTHASH).rlib\
# 	 /librustc_unicode-$(RUSTHASH).rlib\
# 	 /liballoc_system-$(RUSTHASH).rlib\
# 	 /libpanic_abort-$(RUSTHASH).rlib\
# 	 /libunwind-$(RUSTHASH).rlib\
# 	 /liblibc-$(RUSTHASH).rlib)

LIBDRYAD=libdryad.a

LINK_ARGS := -pie --gc-sections -Bsymbolic --dynamic-list=${ETC}/dynamic-list.txt -I${PT_INTERP} -soname ${SONAME} -L$(USRLIB) -nostdlib -e _start

dryad.so.1: $(OUT_DIR)/$(LIBDRYAD)
	@printf "\33[0;4;33mlinking:\33[0m \33[0;32m$(SONAME)\33[0m with $(HASH)\n"
	$(LD) ${LINK_ARGS}\
	 -o ${SONAME}\
	 ${OUT_DIR}/$(LIBDRYAD)\
	 ${RUSTLIBS}\
	 ${CARGO_DEPS}
	cp ${SONAME} /tmp

$(OUT_DIR)/$(LIBDRYAD): $(SRC)
	@printf "\33[0;4;33mcompiling:\33[0m \33[1;32mdryad\33[0m\n"
	CC=$(CC) AR=$(AR) $(CARGO) rustc $(COLOR) -vv --verbose --target=$(TRIPLE) --lib -j 4 -- -C panic=abort #-C lto # uncomment this when lto is important

#almost... but cargo/rustc refuses to compile dylibs with a musl target
#link-args="-Wl,-pie,-I${PT_INTERP},-soname ${SONAME}, --gc-sections, -L${LIB}, -Bsymbolic, -nostdlib, -e _start, -o ${SONAME}, start.o, dryad.o, ${RUSTLIB}/libstd-$(RUSTHASH).rlib, ${RUSTLIB}/libcore-$(RUSTHASH).rlib, ${RUSTLIB}/librand-$(RUSTHASH).rlib, ${RUSTLIB}/liballoc-$(RUSTHASH).rlib, ${RUSTLIB}/libcollections-$(RUSTHASH).rlib, ${RUSTLIB}/librustc_unicode-$(RUSTHASH).rlib, ${RUSTLIB}/liballoc_system-$(RUSTHASH).rlib, ${RUSTLIB}/libcompiler-rt.a, ${RUSTLIB}/liblibc-$(RUSTHASH).rlib, ${CARGO_DEPS}"

clean:
	$(CARGO) clean
	rm ${SONAME}

run: dryad.so.1
	./dryad.so.1

TESTDIR=test
TESTS=$(wildcard ${TESTDIR}/*.c)

tests: ${TESTS}
	@echo "Building regular binary ${TESTDIR}/test with libm and libc"
	$(CC) $(CCOPT) -Wl,-I,${PT_INTERP} ${TESTDIR}/test.c -o ${TESTDIR}/test -lm
	$(CC) $(CCOPT) ${TESTDIR}/test.c -o ${TESTDIR}/ldtest -lm
	@echo "Building thread local binary ${TESTDIR}/tlocal with pthreads and libc"
	$(CC) $(CCOPT) -Wl,-I,${PT_INTERP} ${TESTDIR}/tlocal.c -o ${TESTDIR}/tlocal -lpthread
	$(CC) $(CCOPT) ${TESTDIR}/tlocal.c -o ${TESTDIR}/ldtlocal -lpthread
	@echo "Building complicated binary ${TESTDIR}/snappy linked with libm and snappy (on my system snappy uses libstdc++.so.6, libm.so.6, libc.so.6, and libgcc_s.so.1)"
	$(CC) $(CCOPT) -Wl,-I,${PT_INTERP} ${TESTDIR}/snappy.c -o ${TESTDIR}/snappy -lm -lsnappy
	$(CC) $(CCOPT) ${TESTDIR}/snappy.c -o ${TESTDIR}/ldsnappy -lm -lsnappy

	@echo "Building a binary ${TESTDIR}/float with libm that performs no printing"
	$(CC) $(CCOPT) -Wl,-I,${PT_INTERP} ${TESTDIR}/float.c -o ${TESTDIR}/float -lm
	$(CC) $(CCOPT) ${TESTDIR}/float.c -o ${TESTDIR}/ldfloat -lm

# for testing, debugging, etc.

link:
	@printf "\33[0;4;33mlinking:\33[0m \33[0;32m$(SONAME)\33[0m with $(HASH)\n"
	$(LD) -Map=${ETC}/dryad.map\
	 ${LINK_ARGS}\
	 -o ${SONAME}\
	 ${OUT_DIR}/$(LIBDRYAD)\
	 ${RUSTLIBS}\
	 ${CARGO_DEPS}
#	 -lc -lm

# make moves all day e'er day
moves:
	cp dryad.so.1 /tmp

relocs:
	@objdump -R dryad.so.1 | wc -l

.PHONY: moves link clean tests run relocs
