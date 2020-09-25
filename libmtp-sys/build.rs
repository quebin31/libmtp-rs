const MIN_LIBMTP_VERSION: &str = "1.1.15";

fn main() {
    println!("cargo:rerun-if-changed=libmtp.h");
    if let Err(err) = pkg_config::Config::new()
        .atleast_version(MIN_LIBMTP_VERSION)
        .cargo_metadata(true)
        .probe("libmtp") 
        
    {

        eprintln!("Couldn't find libmtp on your system!  (minimum version: 1.1.15)");
        eprintln!("This crates requires that it's installed and its pkg-config is");
        eprintln!("working correctly!");
        panic!(
            "Couldn't find libmtp via `pkg-config`: {:?}\nPKG_CONFIG_SYSROOT_DIR={}",
            err,
            std::env::var("PKG_CONFIG_SYSROOT_DIR").unwrap_or_default(),
        );
    } 

    let bindings = bindgen::Builder::default()
        .header("libmtp.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .whitelist_function("^LIBMTP_.*")
        .whitelist_var("^LIBMTP_.*")
        .whitelist_type("^LIBMTP_.*")
        .blacklist_type("^__.*")
        .blacklist_type("time_t")
        .blacklist_item("timeval")
        .raw_line("pub type time_t = libc::time_t;")
        .raw_line("pub type timeval = libc::timeval;")
        .generate()
        .expect("Unable to generate bindings");

    bindings
        .write_to_file("src/bindings.rs")
        .expect("Couldn't write bindings");
}
