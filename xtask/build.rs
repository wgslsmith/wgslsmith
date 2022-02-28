use std::env;

fn main() {
    let build_target = env::var("TARGET").unwrap();
    println!("cargo:rustc-env=XTASK_HOST_TARGET={build_target}");
}
