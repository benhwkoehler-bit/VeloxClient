fn main() {
    #[cfg(windows)]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("icons/icon.ico");

        let version = env!("CARGO_PKG_VERSION");
        res.set("CompanyName", "VeloxClient");
        res.set("ProductName", "VeloxClient");
        res.set(
            "FileDescription",
            "VeloxClient (based on OneClient by Polyfrost)",
        );
        res.set(
            "LegalCopyright",
            "GPL-3.0 · based on OneClient © 2026 Polyfrost Inc.",
        );
        res.set("OriginalFilename", "oneclient_app.exe");
        res.set("InternalName", "oneclient_app");
        res.set("FileVersion", version);
        res.set("ProductVersion", version);

        res.compile()
            .expect("failed to embed Windows icon resource");
    }
}
