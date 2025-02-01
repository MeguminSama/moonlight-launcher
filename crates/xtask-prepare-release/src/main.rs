use std::io;
use std::process::Command;

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

        Command::new("makensis.exe")
            .current_dir(&nsis_dir)
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
