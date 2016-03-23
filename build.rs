extern crate gcc;

fn main () {
    
    gcc::compile_library("libstart.a", &["src/arch/x86/asm.s"]);

    //target.x86_64-unknown-linux-musl.dryad
    /*
    println!("cargo:rustc-link-search=static={}", "musldist/lib");
    println!("cargo:rustc-link-lib=static=resolv");
    println!("cargo:rustc-link-lib=static=unwind");
    println!("cargo:rustc-link-lib=static=m");
    println!("cargo:rustc-link-lib=static=c");
    */
//    println!("cargo:rustc-flags=-C hi there");
}
