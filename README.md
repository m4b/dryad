# Welcome

![dryad](https://en.wikipedia.org/wiki/Dryad#/media/File:Dryad11.jpg)

`dryad` is the **first** _parallel_, 64-bit ELF dynamic linker, written from scratch in Rust, and is:

1. not ready for production
2. a prototype
3. doesn't really work
4. in a massive state of flux

but all of these things will disappear in time!

# Build

The provided build script should work; you need:

- `clang` (or `gcc`, just change the clang lines to gcc)
- `rustc` nightly
- `ld`
- an x86-64 linux box

For testing, you can run `run.sh` which compiles `dryad`, copies it to `/tmp` and runs a simple binary `test/test`, whose `PT_INTERPRETER` is `/tmp/dryad.so.1`

# Contributing

Contributions wholeheartedly welcome!  I'd like this to be very much a community project to build a really great, community, production dynamic linker for use in x86-64 (and beyond) linux systems.

If you don't know anything about dynamic linking, that's totally ok!  Here are some resources if you're curious:

1. [The ELF specification](http://flint.cs.yale.edu/cs422/doc/ELF_Format.pdf)
2. [x86-64 System V Application Binary Interface](http://www.x86-64.org/documentation/abi.pdf)
3. [google's bionic dynamic linker source code](http://github.com/android/platform_bionic/)
4. [glibc dynamic linker source code](https://fossies.org/dox/glibc-2.22/rtld_8c_source.html)
5. [sco dynamic linking document](http://www.sco.com/developers/gabi/latest/ch5.dynamic.html)
6. [iecc dynamic linking article](http://www.iecc.com/linker/linker10.html)
7. [ELF loading tutorial](http://www.gelato.unsw.edu.au/IA64wiki/LoadingELFFiles)
8. `man ld-so` for dynamic linking basics
9. `man dlopen` for runtime dynamic linking basics
10. `man 3 getauxval` for information on the auxiliary vector passed by the kernel to programs


I don't have any hard and fast rules on contributing (probably no one will, because does anyone care/know about program interpreters anymore?), but from my past experience contributing to open-source projects, for any **non-minor** changes from a _new_ contributor please first raise a simple issue about:

1. what you want to change
2. why

and then we can discuss it; this way no one's precious time and energy is wasted (basically yours, since you're the one that will have done the ostentible coding work), especially if massive codebase changes are made and it's decided this isn't the best approach.

# TODO:

1. **MAJOR**: `/etc/ld.so.cache` loader and parser
2. **MAJOR**: `dlfcn.h` implementation and shared object bindings for runtime dynamic loading support
3. **MAJOR**: properly init dynamic linker's static musl libc
4. **MAJOR**: someone figure out how to get cargo working + tests + deps + linking, because that would be so, so amazing
4. make unsafe code safer with rust best practices; rust experts definitely needed!
5. add profiling configs
6. add tests
7. implement dynamic linking
8. x all the things
