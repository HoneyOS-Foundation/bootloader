pub mod rootfs;
pub mod tasks;
pub mod tui;

use std::time::Duration;

use hapi::{
    display::Display,
    fs::{fslabel::FsLabel, RamFileSystem},
    js_console::JsConsoleLogger,
};
use rootfs::{extract_rootfs, fetch_rootfs};
use tasks::TaskExecutor;
use tui::Tui;

#[hapi::main]
fn main() -> anyhow::Result<()> {
    JsConsoleLogger::init();
    Display::assume_control()?;

    // Initialize the terminal ui
    Tui::init_once();
    Tui::start();

    // Setup the task manager
    TaskExecutor::init_once();

    TaskExecutor::register("Mounting ramdisk at C:/", || {
        RamFileSystem::init(FsLabel::C).unwrap();
        hapi::process::set_cwd("C:/");
        Tui::log("Process CWD is now C:");
        true
    });
    TaskExecutor::register("Fetching rootfs.zip", || fetch_rootfs().is_ok());
    TaskExecutor::register("Extracting rootfs.zip", || extract_rootfs().is_ok());
    TaskExecutor::start();

    std::thread::sleep(Duration::from_millis(500));
    Display::release_control()?;
    Ok(())
}
