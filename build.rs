fn main() -> std::io::Result<()> {
    #[cfg(windows)]
    {
        winresource::WindowsResource::new()
            .set_icon("./installers/NSIS/assets/icon.ico")
            .compile()?;

        let product_version = format!(
            "{}.{}.{}.0",
            env!("CARGO_PKG_VERSION_MAJOR"),
            env!("CARGO_PKG_VERSION_MINOR"),
            env!("CARGO_PKG_VERSION_PATCH")
        );

        // Automatically update the PRODUCT_VERSION in headers.nsh for the NSIS installer
        let headers = include_str!("./installers/NSIS/headers.nsh");

        let headers: String = headers
            .lines()
            .map(|line| {
                if line.contains("!define PRODUCT_VERSION") {
                    format!("!define PRODUCT_VERSION \"{}\"", product_version)
                } else {
                    line.to_string()
                }
            })
            .collect::<Vec<String>>()
            .join("\n");

        std::fs::write("./installers/NSIS/headers.nsh", headers)?;
    }

    Ok(())
}
