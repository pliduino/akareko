fn main() -> Result<(), Box<dyn std::error::Error>> {
    let libtorrent = vcpkg::Config::new().find_package("libtorrent").unwrap();

    let mut cxx = cxx_build::bridge("src/ffi.rs");

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();

    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        cxx.define("_WIN32_WINNT", Some("0x0A00"));
        cxx.flag_if_supported("/EHsc");
    }

    cxx.define("TORRENT_NO_DEPRECATE", Some("1"));

    cxx.file("src/lt.cpp")
        .std("c++14")
        .include(&manifest_dir)
        .include(manifest_dir + "/target/cxxbridge/lt-rs")
        .includes(libtorrent.include_paths)
        .compile("ltbridge");

    println!("cargo:rerun-if-changed=src/lt.cpp");
    println!("cargo:rerun-if-changed=src/lt.h");
    println!("cargo:rerun-if-changed=src/ffi.rs");

    Ok(())
}
