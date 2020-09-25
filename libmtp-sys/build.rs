fn main() {
    println!("cargo:rustc-link-lib=mtp");
    println!("cargo:rerun-if-changed=libmtp.h");

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
