use std::io;
use std::process::Command;

/// Read the launcher's version from the workspace Cargo.toml, e.g. `0.1.11` -> `0.1.11.0`
/// (the 4-component form Windows version resources and NSIS expect).
#[cfg(windows)]
fn launcher_version(workspace_root: &std::path::Path) -> io::Result<String> {
    let manifest = std::fs::read_to_string(workspace_root.join("Cargo.toml"))?;

    let version = manifest
        .lines()
        .find_map(|line| {
            let line = line.trim();
            line.strip_prefix("version = ")
                .map(|v| v.trim_matches('"').to_string())
        })
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "no `version = \"...\"` found in Cargo.toml",
            )
        })?;

    Ok(format!("{version}.0"))
}

fn main() -> io::Result<()> {
    Command::new("cargo")
        .arg("build")
        .arg("--release")
        .arg("--all")
        .status()?;

    #[cfg(windows)]
    {
        let workspace_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let workspace_root = workspace_root.ancestors().nth(2).unwrap().to_path_buf();

        let installers_dir = workspace_root.join("installers");
        let nsis_dir = installers_dir.join("NSIS");

        let product_version = launcher_version(&workspace_root)?;

        Command::new("makensis.exe")
            .current_dir(&nsis_dir)
            .arg(format!("-DPRODUCT_VERSION={product_version}"))
            .arg("installer.nsi")
            .status()?;

        std::fs::create_dir_all(workspace_root.join("target").join("dist"))?;

        std::fs::copy(
            nsis_dir.join("moonlight installer.exe"),
            workspace_root
                .join("target")
                .join("release")
                .join("moonlight installer.exe"),
        )?;
    }

    Ok(())
}
