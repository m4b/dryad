# Welcome

![dryad](doc/dryad.jpg)

`dryad` is the **first** and **only** _parallel_, 64-bit ELF dynamic linker for GNU/Linux, written from scratch in Rust, and is:

1. not ready for production
2. a prototype
3. doesn't really work
4. in a massive state of flux
5. parallel might be a) impossible, b) not performant, but it will be interesting to try

but ~~all~~ most of these things will disappear in time!

# Build

To get up and running actually requires some work right now, because ~~`rustc` isn't capable of generating stand-alone, completely statically linked binaries, which is a hard and fast requirement of a dynamic linker.  See this [issue](https://internals.rust-lang.org/t/static-binary-support-in-rust/2011) and this great chapter about [linking details](https://doc.rust-lang.org/book/advanced-linking.html).~~ experiencing some issues as elaborated [here](https://internals.rust-lang.org/t/static-binary-support-in-rust/2011/55)

So, I've created a script, `make_static.sh`, basically lifted from the chapter on linking, to "auto-build" a `rustc` compiler for static executables using [musl libc](http://www.musl-libc.org/).  This will take approximately 30 minutes to an hour to download everything (depending on connection obviously) and then compile Rust from source.  Be patient, grab a coffee (or tea).  Or don't, and stare at the output like I do.

In order to build dryad you'll need your typical build tools on a linux system, which varies from distro to distro.  But essentially you'll need:

- `gcc` (or `clang`)
- `ld` (or `ld.gold`)
- `curl`
- an internet connection
- an x86-64 linux box

After that's settled, the following sequence should get you on your way:

1. `git clone http://github.com/m4b/dryad`
2. `cd dryad`
3. `./make_static.sh`
4. **wait 30+ minutes**
5. `./make_tests.sh`
5. `./static.sh`
6. `test/test`

The last script, `./static.sh` does four things:

a. compiles the x86-64 asm stubs which dryad needs (change the `gcc` call to `clang` here if you like)
   `gcc -fPIC -c -o start.o src/arch/x86/asm.s`
b. compiles dryad into an object file using the rust compiler you built 30 minutes ago: `musldist/bin/rustc --target=x86_64-unknown-linux-musl src/lib.rs -g --emit obj -o dryad.o`
c. links the asm stubs with dryad and then the rust standard libs, and pthreads and libc and etc.
d. copies the resulting binary, `dryad.so.1`, into `/tmp/dryad.so.1` because that's what `PT_INTERPRETER` is set to in the test binaries. In the future we'll obviously make this `/usr/lib/dryad.so.1`, or wherever the appropriate place for the dynamic linker is (GNU's is called `ld-linux-x86-64.so.2` btw).

Finally, the last step, running `test/test`, which is a binary generated via `make_tests.sh`, will output a ton of information and then segfault your machine, or perhaps not run at all, or really do any number of things --- I really can't say, since I've only tested on a single machine so far.

`dryad` _should_ be capable of interpreting itself, which you can verify by invoking `./dryad.so.1`.

Eventually I will get around to creating a makefile (or better yet, cargo) --- sorry about that!  Really, stage `c` from above is the problem in the cargo pipeline, and if someone could figure that out, I'd be massively grateful.  I think the only solution, do to the intimate needs of dryad, is to create a cargo subcommand :/

# Contributing

Contributions wholeheartedly welcome!  Let's build a production dynamic linker in Rust for use in x86-64 GNU/Linux systems (and beyond)!  Or not, that's cool too.

If you don't know anything about dynamic linking on x86-64 GNU systems for ELF, that's totally OK, because as far as I can tell, **no one** really does anymore. Here are some random resources if you're curious:

1. [The ELF specification](http://flint.cs.yale.edu/cs422/doc/ELF_Format.pdf)
2. [x86-64 System V Application Binary Interface](http://www.x86-64.org/documentation/abi.pdf)
3. [ELF TLS spec](http://people.redhat.com/aoliva/writeups/TLS/RFC-TLSDESC-x86.txt)
3. [google's bionic dynamic linker source code](http://github.com/android/platform_bionic/)
4. [glibc dynamic linker source code](https://fossies.org/dox/glibc-2.22/rtld_8c_source.html)
5. [sco dynamic linking document](http://www.sco.com/developers/gabi/latest/ch5.dynamic.html)
6. [iecc dynamic linking article](http://www.iecc.com/linker/linker10.html)
7. [ELF loading tutorial](http://www.gelato.unsw.edu.au/IA64wiki/LoadingELFFiles)
8. [Info on the GOT[0] - GOT[2] values](http://users.eecs.northwestern.edu/~kch479/docs/notes/linking.html)
8. `man ld-so` for dynamic linking basics
9. `man dlopen` for runtime dynamic linking basics
10. `man 3 getauxval` for information on the auxiliary vector passed by the kernel to programs
11. I'll also hopefully add a couple articles on some of my _mis_adventures on my essentially [defunct blog](http://www.m4b.io)

# TODOs

Here are some major todos off the top of my head

1. **MAJOR**: `/etc/ld.so.cache` loader and parser
2. **MAJOR**: `dlfcn.h` implementation and shared object bindings for runtime dynamic loading support
3. **MAJOR**: properly init dynamic linker's TLS.  This terrifies me.
4. **MAJOR**: someone figure out how to get cargo working + tests + deps + linking, because that would be so, so amazing
5. implement the GNU bloom filter for dem speeds
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
