//! Types related to task management

use super::TaskContext;
use crate::config::MAX_SYSCALL_NUM; // my code

/// The task control block (TCB) of a task.
#[derive(Copy, Clone)]
pub struct TaskControlBlock {
    /// The task status in it's lifecycle
    pub task_status: TaskStatus,
    /// The task context
    pub task_cx: TaskContext,

    // my code
    /// sys call info
    pub syscall_info: [u32; MAX_SYSCALL_NUM],
    /// first run tiom
    pub first_run: usize,
    // my code
}

/// The status of a task
#[derive(Copy, Clone, PartialEq)]
pub enum TaskStatus {
    /// uninitialized
    UnInit,
    /// ready to run
    Ready,
    /// running
    Running,
    /// exited
    Exited,
}
