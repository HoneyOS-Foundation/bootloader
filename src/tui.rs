use std::sync::{Arc, Once, RwLock, RwLockReadGuard, RwLockWriteGuard};

use hapi::display::Display;

use crate::tasks::TaskExecutor;

/// The frames for the loading icon
const LOADING_FRAMES: &[&str] = &[
    "\x1b[97m[\x1b[91m*\x1b[31m*\x1b[97m    ]\x1b[97m",
    "\x1b[97m[\x1b[31m*\x1b[91m*\x1b[31m*\x1b[97m   ]",
    "\x1b[97m[ \x1b[31m*\x1b[91m*\x1b[31m*\x1b[97m  ]",
    "\x1b[97m[  \x1b[31m*\x1b[91m*\x1b[31m*\x1b[97m ]",
    "\x1b[97m[   \x1b[31m*\x1b[91m*\x1b[31m*\x1b[97m]",
    "\x1b[97m[    \x1b[31m*\x1b[91m*\x1b[97m]",
    "\x1b[97m[   \x1b[31m*\x1b[91m*\x1b[31m*\x1b[97m]",
    "\x1b[97m[  \x1b[31m*\x1b[91m*\x1b[31m*\x1b[97m ]",
    "\x1b[97m[ \x1b[31m*\x1b[91m*\x1b[31m*\x1b[97m  ]",
    "\x1b[97m[\x1b[31m*\x1b[91m*\x1b[31m*\x1b[97m   ]",
];

/// The OK frame for the status indicator
const OK_FRAME: &str = "\x1b[97m[  \x1b[32mok  \x1b[97m]";
// The FAIL frame for the statuc indicator
const FAIL_FRAME: &str = "\x1b[97m[ \x1b[91mfail \x1b[97m]";

// The delay between frames
const FRAME_DELAY: f64 = 0.1;

/// The static state of the terminal
static mut TUI_STATE: Option<Arc<RwLock<TuiState>>> = None;

/// The terminal ui for the bootloader
pub struct Tui;

/// The state of the terminal ui
struct TuiState {
    log: String,
    running: bool,
}

impl Tui {
    /// Initialize the terminal ui
    pub fn init_once() {
        static SET_HOOK: Once = Once::new();
        SET_HOOK.call_once(|| unsafe {
            TUI_STATE = Some(Arc::new(RwLock::new(TuiState {
                log: String::new(),
                running: false,
            })))
        });
    }

    /// Write a string to the log
    pub fn log(string: impl Into<String>) {
        let string: String = string.into();
        let mut state = TuiState::writer();
        let time = hapi::time::system();
        state.log.push_str(&format!(
            "\n{}: {}",
            humantime::format_rfc3339(time),
            string
        ));
    }

    /// Register that a task is completed.
    /// Do nothing if the task is invalid, or is not actually completed
    pub fn register_completed(id: u32) {
        let Some(info) = TaskExecutor::info(id) else {
            return;
        };
        let Some(success) = info.success() else {
            return;
        };
        let success_frame = if success { OK_FRAME } else { FAIL_FRAME };
        let mut state = TuiState::writer();
        state
            .log
            .push_str(&format!("\n{} {}", success_frame, info.descriptor()));
    }

    /// Start a thread that draws to the screen,
    /// will do nothing if the tui is already started.
    pub fn start() {
        {
            let mut state = TuiState::writer();
            if state.running {
                return;
            }
            state.running = true;
        }
        hapi::thread::spawn(|| {
            let mut running = true;
            let mut frame_index = 0;

            // Keep track of the tasks that have been completed
            let mut time_since_last_frame = hapi::time::since_startup();
            while running {
                // Limit the frame rate
                let current_time = hapi::time::since_startup();
                if current_time - time_since_last_frame < FRAME_DELAY {
                    continue;
                }
                time_since_last_frame = current_time;

                // Draw the loader for the current task
                let log_current_task = draw_current_task(frame_index);
                frame_index += 1;
                if frame_index >= LOADING_FRAMES.len() {
                    frame_index = 0;
                }

                // Draw the log
                let state = TuiState::reader();
                let formatted_log = format_log(&state.log);

                Display::set_text(&format!("\x1b[97m{}{}", formatted_log, log_current_task));
                running = state.running;
            }
        });
    }
}

impl TuiState {
    /// Get the reader for the state.
    /// Blocks until the lock is available
    fn reader<'a>() -> RwLockReadGuard<'a, TuiState> {
        loop {
            let state = unsafe { TUI_STATE.as_ref().unwrap() };
            if let Ok(state) = state.try_read() {
                return state;
            }
        }
    }

    /// Get the writer for the state.
    /// Blocks until the lock is available
    fn writer<'a>() -> RwLockWriteGuard<'a, TuiState> {
        loop {
            let state = unsafe { TUI_STATE.as_ref().unwrap() };
            if let Ok(state) = state.try_write() {
                return state;
            }
        }
    }
}

/// Transform the log string into it's displayed form
fn format_log(log: &str) -> String {
    if log.is_empty() {
        return String::new();
    }

    let lines = log.split("\n").skip(1);
    let mut result = String::new();
    for line in lines {
        result.push_str(&format!("\x1b[37m{}\x1b[97m\n", line));
    }

    result
}

/// Draw the loader for the current task
fn draw_current_task(frame_index: usize) -> String {
    if !TaskExecutor::running() {
        return String::new();
    }
    let Some(task_info) = TaskExecutor::current_info() else {
        return String::new();
    };

    let loading_frame = LOADING_FRAMES[frame_index];

    if task_info.success().is_some() {
        String::new()
    } else {
        format!("{} {}", loading_frame, task_info.descriptor())
    }
}
