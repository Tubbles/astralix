fn main() {
    pkg_config::Config::new().probe("SDL2_ttf").unwrap();
    println!("cargo:rerun-if-changed=build.rs");
}
