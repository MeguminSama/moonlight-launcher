use winapi::um::winuser::{MB_ICONERROR, MB_ICONINFORMATION, MB_ICONWARNING, MB_OK};

#[repr(u32)]
pub enum MessageBoxIcon {
    Error = MB_ICONERROR | MB_OK,
    Info = MB_ICONINFORMATION | MB_OK,
    Warning = MB_ICONWARNING | MB_OK,
}

pub fn messagebox(title: &str, body: &str, icon: MessageBoxIcon) {
    use std::os::windows::ffi::OsStrExt;

    use winapi::um::winuser::MessageBoxW;

    let title = std::ffi::OsStr::new(title)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect::<Vec<_>>();

    let body = std::ffi::OsStr::new(body)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect::<Vec<_>>();

    unsafe {
        MessageBoxW(
            std::ptr::null_mut(),
            body.as_ptr(),
            title.as_ptr(),
            icon as u32,
        );
    }
}

pub fn get_latest_executable(dir: &std::path::Path) -> Result<std::path::PathBuf, String> {
    let dir_name = dir
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or("Failed to get directory name as string")?;

    let target_exe_name = format!("{}.exe", dir_name);

    let files = std::fs::read_dir(dir)
        .map_err(|_| "Failed to read directory")?
        .flatten()
        .collect::<Vec<_>>();

    if !files.iter().any(|f| f.file_name() == "Update.exe") {
        return Err("The provided directory does not contain Update.exe.".into());
    }

    let mut app_dirs: Vec<_> = files
        .iter()
        .filter_map(|f| f.file_name().to_str().map(|s| s.to_string()))
        .filter(|f| f.starts_with("app-"))
        .collect();

    app_dirs.sort_by(|a, b| crate::app_version::compare_app_dirs(a, b));

    for app in app_dirs {
        let app_dir = dir.join(app);

        let Ok(app_files) = std::fs::read_dir(&app_dir) else {
            continue;
        };

        let app_files = app_files.flatten().collect::<Vec<_>>();

        if app_files.iter().any(|f| *f.file_name() == *target_exe_name) {
            return Ok(app_dir.join(target_exe_name));
        }
    }

    Err("Failed to find a valid Discord executable".into())
}
