const MIN_LIBMTP_VERSION: &str = "1.1.15";

fn main() {
    if std::env::var("DOCS_RS").is_ok() {
        // skip binding if building on docs.rs
        return;
    }

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
}
