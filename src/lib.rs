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
use tui::tasks::TaskUi;

#[hapi::main]
fn main() -> anyhow::Result<()> {
    JsConsoleLogger::init();
    Display::assume_control()?;

    // Initialize the tracker ui
    TaskUi::init_once();
    TaskUi::start();

    std::thread::sleep(Duration::from_millis(50)); // Some leeway for the tui thread to spawn

    // Setup the task manager
    TaskExecutor::init_once();

    TaskExecutor::register("Mounting ramdisk at C:/", || {
        RamFileSystem::init(FsLabel::C).unwrap();
        hapi::process::set_cwd("C:/");
        TaskUi::log("Process CWD is now C:");
        true
    });
    TaskExecutor::register("Fetching rootfs.zip", || fetch_rootfs().is_ok());
    TaskExecutor::register("Extracting rootfs.zip", || extract_rootfs().is_ok());
    TaskExecutor::start();

    TaskUi::stop();

    std::thread::sleep(Duration::from_millis(100));
    Display::release_control()?;
    Ok(())
}
