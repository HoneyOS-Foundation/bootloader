use std::io::{Cursor, Read};

use hapi::{
    display::Display,
    fs::{dir::Directory, fslabel::FsLabel, File, RamFileSystem},
    js_console::JsConsoleLogger,
    network::{Request, RequestMethod, RequestStatus},
    process::Process,
};

#[hapi::main]
fn main() -> anyhow::Result<()> {
    JsConsoleLogger::init();

    Display::assume_control()?;

    hapi::println!("Rootfs fetched succesfully");
    hapi::println!("Mounting ramdisk at C:/");
    Display::push_stdout();

    RamFileSystem::init(FsLabel::C)?;

    hapi::stdout::clear_line();
    hapi::println!("Mounted ramdisk at C:/");
    Display::push_stdout();

    extract_rootfs()?;

    hapi::println!("Executing startup process");
    Display::push_stdout();

    startup_process()?;

    Display::release_control()?;
    Ok(())
}

/// Extract rootfs and display the output
pub fn extract_rootfs() -> anyhow::Result<()> {
    hapi::println!("Fetching rootfs");
    Display::push_stdout();

    let request = Request::new("rootfs.zip", RequestMethod::Get, "{}")?;
    request.wait()?;

    if request.status()? == RequestStatus::Fail {
        hapi::println!("\x1b[31mFailed to fetch rootfs\x1b[97m");
        Display::push_stdout();
        return Ok(());
    }

    hapi::stdout::clear_line();
    hapi::println!("\x1b[32mSuccesfully fetched rootfs\x1b[97m");
    Display::push_stdout();

    let bytes = request.data()?;
    let mut cursor = Cursor::new(bytes);
    let mut rootfs = zip::ZipArchive::new(&mut cursor)?;

    for i in 0..rootfs.len() {
        let part = match rootfs.by_index(i) {
            Ok(p) => p,
            Err(e) => {
                hapi::println!("\x1b[31mFailed to read part: {}\x1b[97m", e);
                Display::push_stdout();
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
        Display::push_stdout();
    }

    hapi::process::set_cwd("C:/");

    Ok(())
}

/// Execute startup process
fn startup_process() -> anyhow::Result<()> {
    let boot_process = match File::open("bin/beofetch.wasm") {
        Ok(b) => b,
        Err(e) => {
            hapi::println!("\x1b[31mFailed to read startup binary \x1b[97m: {}", e);
            Display::push_stdout();
            return Ok(());
        }
    };
    Display::push_stdout();
    let boot_process_bin = match boot_process.read_all() {
        Ok(b) => b,
        Err(e) => {
            hapi::println!("\x1b[31mFailed to read startup binary \x1b[97m: {}", e);
            Display::push_stdout();
            return Ok(());
        }
    };
    Display::push_stdout();
    if let Some(_) = Process::spawn_sub(&boot_process_bin) {
        hapi::println!("\x1b[32mSuccesfully spawned startup process\x1b[97m");
        Display::push_stdout();
    } else {
        hapi::println!("\x1b[31mFailed to spawn startup process \x1b[97m");
        Display::push_stdout();
    }

    Ok(())
}
