use std::{ffi::CString, io::Cursor};

use hapi::{
    display::DisplayServer,
    js_console::JsConsoleLogger,
    network::{Request, RequestMethod, RequestStatus},
};

#[hapi::main]
fn main() -> anyhow::Result<()> {
    JsConsoleLogger::init();

    let mut display = DisplayServer::register();
    DisplayServer::claim(&display)?;

    display.set_text("Fetching rootfs")?;
    let request = Request::new("rootfs.zip", RequestMethod::Get, "{}")?;
    request.wait()?;

    if request.status()? == RequestStatus::Fail {
        display.set_text("Failed to fetch rootfs")?;
        return Ok(());
    }

    let bytes = request.data()?;
    let mut cursor = Cursor::new(bytes);
    let mut rootfs = zip::ZipArchive::new(&mut cursor)?;

    let mut files: Vec<String> = vec![];
    let mut directories: Vec<String> = vec![];

    for i in 0..rootfs.len() {
        let part = match rootfs.by_index(i) {
            Ok(part) => part,
            Err(e) => {
                log::error!("{}", e);
                continue;
            }
        };
        let name = part.name();
        if part.is_file() {
            files.push(name.to_string());
            continue;
        }
        if part.is_dir() {
            directories.push(name.to_string());
        }
    }

    display.set_text(format!(
        "Fetched rootfs: \n - Files: {:?}\n - Directories: {:?}",
        files, directories
    ))?;

    unsafe {
        let result = hapi::ffi::hapi_fs_init_ramfs('C' as u8);
        if result < 0 {
            log::error!("Failed to initialize ramfs: {}", result);
            return Err(anyhow::anyhow!("Failed to initialize ramfs: {}", result));
        }

        let path = CString::new("C:/hi.txt").unwrap();
        let result = hapi::ffi::hapi_fs_file_create(path.as_ptr() as *const u8);
        if result < 0 {
            log::error!("Failed to create file: {}", result);
            return Err(anyhow::anyhow!("Failed to create file: {}", result));
        }

        let mut id = vec![0 as u8; 37];
        let result = hapi::ffi::hapi_fs_file_get(path.as_ptr() as *const u8, &mut id[0] as *mut u8);
        let id = CString::from_vec_with_nul(id).unwrap();

        if result < 0 {
            log::error!("Failed to get file id!: {}", result);
            return Err(anyhow::anyhow!("Failed to get file id: {}", result));
        }

        let contents = "Hello File system!".to_string();
        let contents_cstring = CString::new(contents.clone()).unwrap();
        let result = hapi::ffi::hapi_fs_file_write(
            'C' as u8,
            id.as_ptr() as *const u8,
            0,
            contents.chars().count() as u32 + 1,
            contents_cstring.as_ptr() as *const u8,
        );

        if result < 0 {
            log::error!("Failed to write to file: {}", result);
            return Err(anyhow::anyhow!("Failed to write to file: {}", result));
        }

        let mut read_contents = vec![0 as u8; contents.chars().count() + 1];
        let result = hapi::ffi::hapi_fs_file_read(
            'C' as u8,
            id.as_ptr() as *const u8,
            0,
            contents.chars().count() as u32 + 1,
            &mut read_contents[0] as *mut u8,
        );

        if result < 0 {
            log::error!("Failed to read from file: {}", result);
            return Err(anyhow::anyhow!("Failed to read from file: {}", result));
        }

        log::info!("{:?}", CString::from_vec_with_nul(read_contents));
    }

    Ok(())
}
