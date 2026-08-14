fn main() {
    #[cfg(target_os = "macos")]
    if let Ok(developer_dir) = std::process::Command::new("xcode-select")
        .arg("-p")
        .output()
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_owned())
    {
        let command_line_tools_swift =
            std::path::Path::new(&developer_dir).join("usr/lib/swift/macosx");
        if command_line_tools_swift.is_dir() {
            println!(
                "cargo:rustc-link-search=native={}",
                command_line_tools_swift.display()
            );
        }
    }
    tauri_build::build();
}
