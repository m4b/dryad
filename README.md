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

but ~~all~~ most of these things will disappear in time!

# Build

Now that [PIC musl has landed](https://internals.rust-lang.org/t/static-binary-support-in-rust/2011/55), setting up a build environment (on Linux) is easy.  

In order to build dryad you'll need your typical build tools on a linux system, which varies from distro to distro.  But essentially you'll need:

- `gcc` (or `clang`)
- `ld` (or `ld.gold`)
- `curl`
- an internet connection
- an x86-64 linux box

Once that's settled, you have two options, both fairly easy.

## Setup - Easy

Just run `./setup.sh`, which will download and install the latest rust nightly into the `rust` directory.

You can then proceed as normal:

1. `./gen_tests.sh` - builds the test binaries (do this once)
2. `./make.sh` - compiles `dryad.so.1` and copies it to `/tmp`
3. `test/test` - runs the test binary `test`, whose `PT_INTERPRETER` is `/tmp/dryad.so.1`

## Setup - Use Nightly Rust, slightly less easier

If the latest `rustc` is not installed, just download and use the latest version of rustup script to install the latest nightly with the musl target enabled, i.e.:

```
curl -sSf https://static.rust-lang.org/rustup.sh > rustup.sh
chmod +x rustup.sh
sudo ./rustup.sh --channel=nightly --with-target=x86_64-unknown-linux-musl
```

And then edit `make.sh` script to use `/usr/local` as the `$PREFIX` instead of `rust`.

You can then compile and run as normal.

## Why `make.sh` (and not `cargo`)

The last script, `make.sh`, does four things:

1. compiles the x86-64 asm stubs which dryad needs (change the `gcc` call to `clang` here if you like) `gcc -fPIC -c -o start.o src/arch/x86/asm.s`
2. compiles dryad into an object file: `rustc --target=x86_64-unknown-linux-musl src/lib.rs -g --emit obj -o dryad.o`
3. links the asm stubs with dryad and then the rust standard libs, and pthreads and libc and etc., and provides the very important linker flags such as `-pie`, `-Bsymbolic`, `-I/tmp/dryad.so.1`, `-soname dryad.so.1`, etc.
4. copies the resulting binary, `dryad.so.1`, into `/tmp/dryad.so.1` because that's what `PT_INTERPRETER` is set to in the test binaries. In the future we'll obviously make this `/usr/lib/dryad.so.1`, or wherever the appropriate place for the dynamic linker is (GNU's is called `ld-linux-x86-64.so.2` btw).

Eventually I will get around to creating a makefile (or better yet, cargo) --- sorry about that!  Really, stage `1` and `3` from above is the problem in the cargo pipeline, and if someone could figure that out, I'd be massively grateful.  I think the only solution, due to the intimate needs of dryad, is to create a cargo subcommand :/

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

1. **MAJOR**: `/etc/ld.so.cache` loader and parser
2. **MAJOR**: `dlfcn.h` implementation and shared object bindings for runtime dynamic loading support
3. **MAJOR**: properly init dynamic linker's TLS.  This terrifies me.
4. **MAJOR**: someone figure out how to get cargo working + tests + deps + linking, because that would be so, so amazing
5. add the `rtld_dl_activity` gdb/debugger calls for notifying gdb, et. al when shared libraries are loaded, etc.  This will make debugging lazy plt calls _significantly_ easier.
5. ~~implement the GNU bloom filter for dem speeds~~
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
