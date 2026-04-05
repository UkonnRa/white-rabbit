fn main() {
    tauri_build::build();

    // Workaround: tauri-build embeds a Common Controls v6 manifest into the
    // main binary (via rustc-link-arg-bins), but test executables don't get it.
    // Without the manifest Windows loads comctl32 v5, which is missing
    // TaskDialogIndirect, causing STATUS_ENTRYPOINT_NOT_FOUND (0xc0000139).
    //
    // We fix this by passing the same manifest to the MSVC linker for test
    // targets using cargo:rustc-link-arg-tests.
    //
    // Upstream: https://github.com/tauri-apps/tauri/issues/13419
    #[cfg(windows)]
    {
        let manifest = std::path::Path::new("tests/comctl32-v6.exe.manifest")
            .canonicalize()
            .expect("comctl32-v6.exe.manifest not found");
        println!("cargo:rustc-link-arg-tests=/MANIFEST:EMBED");
        println!(
            "cargo:rustc-link-arg-tests=/MANIFESTINPUT:{}",
            manifest.display()
        );
    }
}
