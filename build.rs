use std::path::*;
use std::env;
use std::process::Command;

extern crate gcc;

fn main () {
    
    let out_dir = env::var("OUT_DIR").unwrap();
    let base = env::current_dir().unwrap();
    //.object
    //.flag
    gcc::compile_library("libstart.a", &["src/arch/x86/asm.s"]);
/*
    gcc::Config::new()
        .file(Path::new(&base).join("src").join("arch").join("x86").join("asm.s"))
        .flag("-c")
        .compile("libstart.a");
*/
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
