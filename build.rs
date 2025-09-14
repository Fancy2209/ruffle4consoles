fn main() {
    #[cfg(target_os = "horizon")]
    println!("cargo:rerun-if-changed=build.rs");
   
    #[cfg(target_os = "horizon")]
    println!("cargo:rustc-link-lib=dylib=nx");
}
