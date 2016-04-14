extern crate gcc;

fn main () {
    gcc::Config::new()
        .file("src/arch/x86/asm.s")
        .target("x86_64-unknown-linux-gnu") // need to set this otherwise complains that we don't have musl-gcc, but we don't care how the asm really gets compiled, just that we have it
        .compile("libstart.a");
}
