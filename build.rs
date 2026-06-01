// Embed icon & metadata ke .exe Windows (muncul di Explorer, taskbar, properties).
fn main() {
    #[cfg(target_os = "windows")]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/icon.ico");
        res.set("ProductName", "Time-Jutsu");
        res.set("FileDescription", "Time-Jutsu — Master your time.");
        res.set("CompanyName", "Time-Jutsu");
        res.set("LegalCopyright", "© 2026 Time-Jutsu");
        if let Err(e) = res.compile() {
            println!("cargo:warning=gagal embed resource icon: {e}");
        }
    }
}
