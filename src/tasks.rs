use std::sync::{Arc, Once, RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::tui::tasks::TaskUi;

/// The task manager state
static mut TASK_MANAGER_STATE: Option<Arc<RwLock<TaskManagerState>>> = None;

/// The function for a task
pub type TaskFn = dyn Fn() -> bool;

/// A task excuted by the boorloader
struct Task {
    descriptor: String,
    callable: Option<Box<TaskFn>>,
    success: Option<bool>,
}

/// The information for the task state
#[derive(Debug, Clone)]
pub struct TaskInfo {
    descriptor: String,
    success: Option<bool>,
}

impl TaskInfo {
    pub fn descriptor(&self) -> &str {
        &self.descriptor
    }

    pub fn success(&self) -> Option<bool> {
        self.success
    }
}

/// The state of the task manager
struct TaskManagerState {
    tasks: Vec<Task>,
    current: u32,
    running: bool,
}

/// Manages tasks for the bootloader
pub struct TaskExecutor;

impl TaskExecutor {
    /// Initialize the terminal ui
    pub fn init_once() {
        static SET_HOOK: Once = Once::new();
        SET_HOOK.call_once(|| unsafe {
            TASK_MANAGER_STATE = Some(Arc::new(RwLock::new(TaskManagerState {
                tasks: Vec::new(),
                current: 0,
                running: false,
            })))
        });
    }

    /// Register a task and return it's id
    pub fn register<F>(descriptor: &str, task: F) -> u32
    where
        F: Fn() -> bool + 'static,
    {
        let mut state = TaskManagerState::writer();
        state.tasks.push(Task {
            descriptor: descriptor.to_string(),
            callable: Some(Box::new(task)),
            success: None,
        });
        state.tasks.len() as u32 - 1
    }

    /// Start executing the tasks
    pub fn start() {
        let count_tasks = {
            let mut state = TaskManagerState::writer();
            state.running = true;
            state.tasks.len()
        };

        for i in 0..count_tasks {
            // Take the callable function from the manager
            let callable = {
                let mut state = TaskManagerState::writer();
                state.running = true;
                let task = state.tasks.get_mut(i).unwrap();
                task.callable.take().unwrap()
            };

            // Execute the task
            let result = callable();

            // Update the task manager
            {
                let mut state = TaskManagerState::writer();
                state.current += 1;
                let task = state.tasks.get_mut(i).unwrap();
                task.success = Some(result);
                state.running = result;
            };

            TaskUi::register_completed(i as u32);

            if !result {
                break;
            }
        }
    }

    /// Check if the task executor is running
    pub fn running() -> bool {
        let state = TaskManagerState::reader();
        state.running
    }

    /// Get the info of the task
    pub fn info(id: u32) -> Option<TaskInfo> {
        let state = TaskManagerState::reader();
        let task = state.tasks.get(id as usize)?;
        Some(TaskInfo {
            descriptor: task.descriptor.clone(),
            success: task.success,
        })
    }

    /// Get the info of the current task
    pub fn current_info() -> Option<TaskInfo> {
        let state = TaskManagerState::reader();
        let current = state.current;
        let task = state.tasks.get(current as usize)?;
        Some(TaskInfo {
            descriptor: task.descriptor.clone(),
            success: task.success,
        })
    }

    /// Get the currently executing state
    pub fn current() -> u32 {
        let state = TaskManagerState::reader();
        state.current
    }

    /// Get the descriptor of a task
    pub fn descriptor(id: u32) -> Option<String> {
        let state = TaskManagerState::reader();
        let task = state.tasks.get(id as usize)?;
        Some(task.descriptor.clone())
    }

    /// Get the completed tasks
    pub fn completed_tasks() -> Vec<u32> {
        let state = TaskManagerState::reader();
        state
            .tasks
            .iter()
            .enumerate()
            .filter(|(_, task)| task.success.is_some())
            .map(|(id, _)| id as u32)
            .collect()
    }
}

impl TaskManagerState {
    /// Get the reader for the state.
    /// Blocks until the lock is available
    fn reader<'a>() -> RwLockReadGuard<'a, TaskManagerState> {
        loop {
            let state = unsafe { TASK_MANAGER_STATE.as_ref().unwrap() };
            if let Ok(state) = state.try_read() {
                return state;
            }
        }
    }

    /// Get the writer for the state.
    /// Blocks until the lock is available
    fn writer<'a>() -> RwLockWriteGuard<'a, TaskManagerState> {
        loop {
            let state = unsafe { TASK_MANAGER_STATE.as_ref().unwrap() };
            if let Ok(state) = state.try_write() {
                return state;
            }
        }
    }
}
