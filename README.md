# Welcome

[![Build Status](https://travis-ci.org/m4b/dryad.svg?branch=master)](https://travis-ci.org/m4b/dryad) [![Floobits Status](https://floobits.com/m4b/dryad.svg)](https://floobits.com/m4b/dryad/redirect) https://gitter.im/m4b/dryad

![dryad](doc/dryad.jpg)

`dryad` is the **first** and **only** _parallel_, 64-bit ELF dynamic linker for GNU/Linux, written from scratch in Rust, and is:

0. not parallel
1. not ready for production
2. a prototype
3. doesn't really work
4. in a massive state of flux
5. parallel might be a) impossible, b) not performant, but it will be interesting to try

~~but ~~all~~ most of these things will disappear in time!~~

Work has stalled on this for a number of reasons, primarily as outlined [here](https://www.google.com/url?q=https%3A%2F%2Fgithub.com%2Fm4b%2Fdryad%2Fissues%2F5%23issuecomment-262696880&sa=D&sntz=1&usg=AFQjCNHKreL2aMzs1xwCuKnF2KMohHwOsw), but I tinker with it from now and then.

I have some ideas to fix things, but so many things to work on!

If you want to contribute, PRs or suggestions, comments, issues, always welcome :) If you want to hack on some other fun binary stuff, [goblin](https://github.com/m4b/goblin) or [cargo-sym](https://github.com/m4b/cargo-sym) could always use an extra hand or two.

# Build

You need to install `rustup` [tool](https://www.rustup.rs/), and then switch to nightly and add the musl target:

```
rustup default nightly
rustup target add x86_64-unknown-linux-musl
```

Of course, you will also need your typical build tools on a linux system, essentially:

- `gcc` (or `clang`)
- `ld` (or `ld.gold`)
- `curl`
- an internet connection
- an x86-64 GNU/Linux box

Unfortunately, I currently do not support cross compiling at the moment (which is an unusual use case anyway), so you will need an x86-64 GNU/Linux machine, otherwise it will fail.

Once that's settled you can then proceed as normal:

1. `./gen_tests.sh` - builds the test binaries (do this once) (will add this as a make target soon)
2. `make` - compiles `dryad.so.1` and copies it to `/tmp`
3. `make run` - runs `./dryad.so.1`, this _should_ run correctly without segfaulting, please file a bug if it does not.
4. `test/test` - runs the test binary `test`, whose `PT_INTERPRETER` is `/tmp/dryad.so.1`

## Compilation and Linking Requirements

The `Makefile` does four things:

1. compiles the x86-64 asm stubs which dryad needs (change the `gcc` call to `clang` here if you like) `gcc -fPIC -c -o start.o src/arch/x86/asm.s`
2. compiles dryad into an object file: `rustc --target=x86_64-unknown-linux-musl src/lib.rs -g --emit obj -o dryad.o`
3. links the asm stubs with dryad and then the rust standard libs, and pthreads and libc and etc., and provides the very important linker flags such as `-pie`, `-Bsymbolic`, `-I/tmp/dryad.so.1`, `-soname dryad.so.1`, etc.
4. copies the resulting binary, `dryad.so.1`, into `/tmp/dryad.so.1` because that's what `PT_INTERPRETER` is set to in the test binaries. In the future we'll obviously make this `/usr/lib/dryad.so.1`, or wherever the appropriate place for the dynamic linker is (GNU's is called `ld-linux-x86-64.so.2` btw).

Really, stage `1` and `3` from above is the problem in the cargo pipeline, which is why I still need to manually link.  Additionally, rustc doesn't like to compile a musl binary as a shared object.

I believe some of these issues will go away if I transfer the start assembly into inline assembly in Rust source code (thereby potentially eliminating step 1), but the musl issue could be a problem.

# Running

The last step, running `test/test` (or any of the other test binaries in `test`), will output a ton of information and then segfault your machine, or perhaps not run at all, or really do any number of things --- I really can't say, since I've only tested on a single machine so far.

**NOTE**: if you're on Ubuntu or another linux distro which doesn't place `libc` in `/usr/lib`, you'll need to pass `LD_LIBRARY_PATH=/path/to/libc` to your `test/test`, i.e.: `LD_LIBRARY_PATH=/path/to/libc test/test`.  Furthermore, if `libc` doesn't have symbolic links for the `soname` pointing to the actual binary, or the actual binary _is_ installed as the `soname`, then it also won't work.  We need `ld.so.cache` reader and parser for this - feel free to work on it!

However, `dryad` is _almost_ capable of interpreting a (simple) binary (like `test/test`) which uses `libc.so.6`.

Specifically, this means is that `dryad` at a high level does the following:

1. relocates itself
2. loads and `mmap`'s all binaries in the flattened dependency list
3. relocates every loaded binary (technically, relocates a subset of the most common relocation symbols)
4. sets up each binary's GOT with its runtime symbol resolution function (`_dryad_resolve_symbol`), and its "rendezvous" data structure
5. resolves GNU ifuncs, and if `LD_BIND_NOW` is set, prebinds all function symbols.
5. passes control to the executable
6. (optionally, if `LD_BIND_NOW` is not set) lazily binds function calls
7. segfaults

There are _several_ major, and _many_ minor tasks that need to be finished to be even remotely "complete".  The first and most major one is properly setting up TLS.  Currently, it hacks it about by just calling the musl symbol `__init_tls` so we don't segfault on `fs:0` accesses and their ilk.

But it really needs to be properly setup, as it's a delicate procedure.

This is easily the least documented part of the entire dynamic linking process I have come across, so work is slow going.  Also there are some questions about how this will work exactly, which I'll detail at some other time, or in a blog post.

Lastly, `dryad` _should_ be capable of interpreting itself, which you can verify by invoking `./dryad.so.1` (yes, dryad is it's own program interpreter).

# Project Goals

### 1. Documenting a Dynamic Linker

The primary goal of this project is to completely document:

1. the dynamic linking _process_ on an GNU/Linux ELF x86-64 system
2. an implementation of such a process

The current state of documentation and information on this subject is an *embarassment*, and I'm continually appalled at the lack of materials, documentation, etc.  I've jokingly told people I'm worried what will happen when all the old C programmers die - but I'm not really joking.

Code is not documentation.  If it were, then this project would have been easy and finished some time ago.

As such, I hope to thoroughly document the implementation, the process, and maybe even my experiences.

**I will be updating this section with more content shortly, please bear with me.**

### 2. Implementing a Dynamic Linker

The current target implementation for dryad is an ELF x86-64 GNU/Linux system.

This is important to note:

1. The ELF loader only supports the 64-bit variant
2. The asm assumes an x86-64 instruction set
3. The linker currently targets Linux, although this need not be set in stone.

I would like to have a working implementation for an ELF x86-64 GNU/Linux target before/if beginning work on other architectures or systems.

That being said, in particular I'm not very interested in porting dryad to work on 32-bit Linux systems, because:

1. 32-bit systems are in obsolescence in my opinion
2. Will _significantly_ complicate the ELF target in the source code, as cfg flags would be needed depending on what target we want to switch at build time, etc.
3. 32-bit ELF dynamic linking is much better documented, and I want to document a 64-bit dynamic linker

# Contributing

Contributions wholeheartedly welcome!  Let's build a production dynamic linker in Rust for use in x86-64 GNU/Linux systems (and beyond)!  Or not, that's cool too.

If you don't know anything about dynamic linking on x86-64 GNU systems for ELF, that's totally OK, because as far as I can tell, **no one** really does anymore. Here are some random resources if you're curious:

1. [The ELF specification](http://flint.cs.yale.edu/cs422/doc/ELF_Format.pdf)
2. [x86-64 System V Application Binary Interface](http://www.x86-64.org/documentation/abi.pdf)
3. [ELF TLS spec](http://people.redhat.com/aoliva/writeups/TLS/RFC-TLSDESC-x86.txt)
3. [google's bionic dynamic linker source code](http://github.com/android/platform_bionic/)
4. [glibc dynamic linker source code](https://fossies.org/dox/glibc-2.22/rtld_8c_source.html)
5. [musl dynlink.c code](http://git.musl-libc.org/cgit/musl/tree/ldso/dynlink.c)
6. [sco dynamic linking document](http://www.sco.com/developers/gabi/latest/ch5.dynamic.html)
7. [iecc dynamic linking article](http://www.iecc.com/linker/linker10.html)
8. [ELF loading tutorial](http://www.gelato.unsw.edu.au/IA64wiki/LoadingELFFiles)
9. [Info on the GOT[0] - GOT[2] values](http://users.eecs.northwestern.edu/~kch479/docs/notes/linking.html)
10. `man ld-so` for dynamic linking basics
11. `man dlopen` for runtime dynamic linking basics
12. `man 3 getauxval` for information on the auxiliary vector passed by the kernel to programs
13. I'll also hopefully add a couple articles on some of my _mis_adventures on my essentially [defunct blog](http://www.m4b.io)

# TODOs

Here are some major todos off the top of my head

1. **MAJOR**: properly init dynamic linker's TLS: it's the _final countdown_.
2. **MAJOR**: `dlfcn.h` implementation and shared object bindings for runtime dynamic loading support
3. **MINOR**: `/etc/ld.so.cache` loader and parser
6. better documentation
7. fix any number of the todos littered across the code
8. make unsafe code safer with rust best practices; rust experts definitely needed!
9. add profiling configs
10. add tests
11. actually implement dynamic linking without segfaulting
12. x all the things

# Coda

Always remember:
> Be _excellent_ to each other
