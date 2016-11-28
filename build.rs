extern crate gcc;

use std::env;

fn main () {

    let target = env::var("TARGET").unwrap();
    println!("target: {}", target);

    let (target, arch) = match target.as_str() {
        "x86_64-unknown-linux-musl" => {
            ("x86_64-unknown-linux-gnu", "x86_64")
        },
        "i686-unknown-linux-musl" => {
            ("i686-unknown-linux-gnu", "x86")
        },
        target => {
            if target.contains("aarch64") {
                (target, "arm64")
            } else if target.contains("arm"){
                (target, "arm")
            } else {
                panic!(format!("Unsupported target architecture: {}", target))
            }
        },
    };

    gcc::Config::new()
        .file(format!("src/arch/{}/asm.s", arch))
        .target(target)
        .compile("libstart.a");
}
