fn main() -> std::io::Result<()> {
    #[cfg(windows)]
    {
        winresource::WindowsResource::new()
            .set_icon("./installers/NSIS/assets/icon.ico")
            .compile()?;
    }

    Ok(())
}
