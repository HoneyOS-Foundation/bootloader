use std::{
    io::{Cursor, Read},
    time::Duration,
};

use hapi::{
    display::{Display, DisplayServer},
    fs::{dir::Directory, fslabel::FsLabel, File, RamFileSystem},
    js_console::JsConsoleLogger,
    network::{Request, RequestMethod, RequestStatus},
    process::{self, Process},
};

#[hapi::main]
fn main() -> anyhow::Result<()> {
    JsConsoleLogger::init();

    let mut display = DisplayServer::register();
    DisplayServer::claim(&display)?;

    hapi::println!("Rootfs fetched succesfully");
    hapi::println!("Mounting ramdisk at C:/");
    display.push_stdout()?;

    RamFileSystem::init(FsLabel::C)?;

    hapi::stdout::clear_line();
    hapi::println!("Mounted ramdisk at C:/");
    display.push_stdout()?;

    extract_rootfs(&mut display)?;

    hapi::println!("Executing startup process");
    display.push_stdout()?;

    startup_process(&mut display)?;
    Ok(())
}

/// Extract rootfs and display the output
pub fn extract_rootfs(display: &mut Display) -> anyhow::Result<()> {
    hapi::println!("Fetching rootfs");
    display.push_stdout()?;

    let request = Request::new("rootfs.zip", RequestMethod::Get, "{}")?;
    request.wait()?;

    if request.status()? == RequestStatus::Fail {
        hapi::println!("\x1b[31mFailed to fetch rootfs\x1b[97m");
        display.push_stdout()?;
        return Ok(());
    }

    hapi::stdout::clear_line();
    hapi::println!("\x1b[32mSuccesfully fetched rootfs\x1b[97m");
    display.push_stdout()?;

    let bytes = request.data()?;
    let mut cursor = Cursor::new(bytes);
    let mut rootfs = zip::ZipArchive::new(&mut cursor)?;

    for i in 0..rootfs.len() {
        let part = match rootfs.by_index(i) {
            Ok(p) => p,
            Err(e) => {
                hapi::println!("\x1b[31mFailed to read part: {}\x1b[97m", e);
                display.push_stdout()?;
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
            hapi::println!("Extracted \"{}\"", path);
            continue;
        }
        if part.is_dir() {
            Directory::create(&path)?;
            hapi::println!("Extracted \"{}\"", path);
        }
        display.push_stdout()?;
    }

    hapi::process::set_cwd("C:/");

    Ok(())
}

/// Execute startup process
fn startup_process(display: &mut Display) -> anyhow::Result<()> {
    let boot_process = match File::open("bin/beofetch.wasm") {
        Ok(b) => b,
        Err(e) => {
            hapi::println!("\x1b[31mFailed to read startup binary \x1b[97m: {}", e);
            display.push_stdout()?;
            return Ok(());
        }
    };
    display.push_stdout()?;
    let boot_process_bin = match boot_process.read_all() {
        Ok(b) => b,
        Err(e) => {
            hapi::println!("\x1b[31mFailed to read startup binary \x1b[97m: {}", e);
            display.push_stdout()?;
            return Ok(());
        }
    };
    display.push_stdout()?;
    if let Some(_) = Process::spawn_sub(&boot_process_bin) {
        hapi::println!("\x1b[32mSuccesfully spawned startup process\x1b[97m");
        display.push_stdout()?;
    } else {
        hapi::println!("\x1b[31mFailed to spawn startup process \x1b[97m");
        display.push_stdout()?;
    }

    Ok(())
}
