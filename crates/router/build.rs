fn main() {
    // Routing a link needs none of these; delay-loading them saves ~2 ms of process startup.
    if std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc") {
        for dll in ["shell32.dll", "combase.dll", "user32.dll", "advapi32.dll"] {
            println!("cargo:rustc-link-arg-bin=broxer=/DELAYLOAD:{dll}");
        }
        println!("cargo:rustc-link-arg-bin=broxer=delayimp.lib");
    }
}
