use std::{
    io::{Cursor, Read},
    ptr::addr_of,
    sync::{Arc, Mutex, Once},
    time::Duration,
};

use hapi::{
    fs::{dir::Directory, File},
    network::{Request, RequestMethod, RequestStatus},
};

use crate::tui::tasks::TaskUi;

/// The rootfs stored in memory
static mut ROOTFS: Option<Arc<Mutex<Vec<u8>>>> = None;

/// Fetch the rootfs
pub fn fetch_rootfs() -> anyhow::Result<()> {
    let request = Request::new("rootfs.zip", RequestMethod::Get, "{}")?;
    request.wait()?;

    if request.status()? == RequestStatus::Fail {
        TaskUi::log("Request exited with status `RequestStatus::Fail`");
        return Err(anyhow::anyhow!(
            "Request exited with status `RequestStatus::Fail`"
        ));
    }

    let data = request.data()?;
    set_rootfs(&data);
    Ok(())
}

/// Extract the rootfs
pub fn extract_rootfs() -> anyhow::Result<()> {
    let rootfs = unsafe { ROOTFS.as_ref().unwrap() };
    let rootfs = rootfs.try_lock().map_err(|e| {
        TaskUi::log(format!("Could not aquire rootfs lock: {}", e));
        anyhow::anyhow!("Could not aquire rootfs lock: {}", e)
    })?;
    let mut cursor = Cursor::new(rootfs.clone());
    let mut rootfs = zip::ZipArchive::new(&mut cursor)?;

    for i in 0..rootfs.len() {
        let part = match rootfs.by_index(i) {
            Ok(p) => p,
            Err(e) => {
                TaskUi::log(format!("\x1b[31mFailed to read part: \x1b[97m{}", e));
                continue;
            }
        };
        let name = part.name();
        let path = format!("C:/{}", name);
        if part.is_file() {
            let mut file = File::create(&path)?;
            let bytes = part.bytes();
            let bytes = bytes
                .filter(|f| f.is_ok())
                .map(|f| f.unwrap())
                .collect::<Vec<u8>>();
            file.write(0, &bytes)?;
            TaskUi::log(format!("Extracted \"{}\"", path));
            continue;
        }
        if part.is_dir() {
            Directory::create(&path)?;
            TaskUi::log(format!("Extracted \"{}\"", path));
        }

        std::thread::sleep(Duration::from_secs_f32(0.2));
    }

    Ok(())
}

/// Set the value of the rootfs in memory.
/// Makes sure the rootfs is set first.
fn set_rootfs(value: &[u8]) {
    static SET_HOOK: Once = Once::new();
    SET_HOOK.call_once(|| unsafe { ROOTFS = Some(Arc::new(Mutex::new(Vec::new()))) });

    // # SAFETY
    // Since the reference is only used once, and there is garunteed to be something at that memory location, we are not accessing bad memory.
    if let Some(rootfs) = unsafe { (*addr_of!(ROOTFS)).clone() } {
        if let Ok(mut rootfs) = rootfs.lock() {
            rootfs.clear();
            rootfs.extend_from_slice(value);
        }
    }
}
