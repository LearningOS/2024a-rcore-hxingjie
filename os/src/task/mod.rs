//! Task management implementation
//!
//! Everything about task management, like starting and switching tasks is
//! implemented here.
//!
//! A single global instance of [`TaskManager`] called `TASK_MANAGER` controls
//! all the tasks in the operating system.
//!
//! Be careful when you see `__switch` ASM function in `switch.S`. Control flow around this function
//! might not be what you expect.

mod context;
mod switch;
#[allow(clippy::module_inception)]
mod task;

use crate::loader::{get_app_data, get_num_app};

use crate::sync::UPSafeCell;
use crate::timer::get_time_ms;
use crate::trap::TrapContext;
use alloc::vec::Vec;
use lazy_static::*;
use switch::__switch;
pub use task::{TaskControlBlock, TaskStatus};

pub use context::TaskContext;

use crate::config::MAX_SYSCALL_NUM;

/// The task manager, where all the tasks are managed.
///
/// Functions implemented on `TaskManager` deals with all task state transitions
/// and task context switching. For convenience, you can find wrappers around it
/// in the module level.
///
/// Most of `TaskManager` are hidden behind the field `inner`, to defer
/// borrowing checks to runtime. You can see examples on how to use `inner` in
/// existing functions on `TaskManager`.
pub struct TaskManager {
    /// total number of tasks
    num_app: usize,
    /// use inner value to get mutable access
    inner: UPSafeCell<TaskManagerInner>,
}

/// The task manager inner in 'UPSafeCell'
struct TaskManagerInner {
    /// task list
    tasks: Vec<TaskControlBlock>,
    /// id of current `Running` task
    current_task: usize,
}

lazy_static! {
    /// a `TaskManager` global instance through lazy_static!
    pub static ref TASK_MANAGER: TaskManager = {
        println!("init TASK_MANAGER");
        let num_app = get_num_app();
        println!("num_app = {}", num_app);
        let mut tasks: Vec<TaskControlBlock> = Vec::new();
        for i in 0..num_app {
            tasks.push(TaskControlBlock::new(get_app_data(i), i));
        }
        TaskManager {
            num_app,
            inner: unsafe {
                UPSafeCell::new(TaskManagerInner {
                    tasks,
                    current_task: 0,
                })
            },
        }
    };
}

impl TaskManager {
    /// Run the first task in task list.
    ///
    /// Generally, the first task in task list is an idle task (we call it zero process later).
    /// But in ch4, we load apps statically, so the first task is a real app.
    fn run_first_task(&self) -> ! {
        let mut inner = self.inner.exclusive_access();
        let next_task = &mut inner.tasks[0];
        next_task.task_status = TaskStatus::Running;

        // my code
        next_task.first_run = get_time_ms();
        // my code

        let next_task_cx_ptr = &next_task.task_cx as *const TaskContext;
        drop(inner);
        let mut _unused = TaskContext::zero_init();
        // before this, we should drop local variables that must be dropped manually
        unsafe {
            __switch(&mut _unused as *mut _, next_task_cx_ptr);
        }
        panic!("unreachable in run_first_task!");
    }

    /// Change the status of current `Running` task into `Ready`.
    fn mark_current_suspended(&self) {
        let mut inner = self.inner.exclusive_access();
        let cur = inner.current_task;
        inner.tasks[cur].task_status = TaskStatus::Ready;
    }

    /// Change the status of current `Running` task into `Exited`.
    fn mark_current_exited(&self) {
        let mut inner = self.inner.exclusive_access();
        let cur = inner.current_task;
        inner.tasks[cur].task_status = TaskStatus::Exited;
    }

    /// Find next task to run and return task id.
    ///
    /// In this case, we only return the first `Ready` task in task list.
    fn find_next_task(&self) -> Option<usize> {
        let inner = self.inner.exclusive_access();
        let current = inner.current_task;
        (current + 1..current + self.num_app + 1)
            .map(|id| id % self.num_app)
            .find(|id| inner.tasks[*id].task_status == TaskStatus::Ready)
    }

    /// Get the current 'Running' task's token.
    fn get_current_token(&self) -> usize {
        let inner = self.inner.exclusive_access();
        inner.tasks[inner.current_task].get_user_token()
    }

    /// Get the current 'Running' task's trap contexts.
    fn get_current_trap_cx(&self) -> &'static mut TrapContext {
        let inner = self.inner.exclusive_access();
        inner.tasks[inner.current_task].get_trap_cx()
    }

    /// Change the current 'Running' task's program break
    pub fn change_current_program_brk(&self, size: i32) -> Option<usize> {
        let mut inner = self.inner.exclusive_access();
        let cur = inner.current_task;
        inner.tasks[cur].change_program_brk(size)
    }

    /// Switch current `Running` task to the task we have found,
    /// or there is no `Ready` task and we can exit with all applications completed
    fn run_next_task(&self) {
        if let Some(next) = self.find_next_task() {
            let mut inner = self.inner.exclusive_access();
            let current = inner.current_task;
            inner.tasks[next].task_status = TaskStatus::Running;
            inner.current_task = next;
            
            // my code
            if inner.tasks[next].first_run == 0 {
                inner.tasks[next].first_run = get_time_ms();
            }
            // my code

            let current_task_cx_ptr = &mut inner.tasks[current].task_cx as *mut TaskContext;
            let next_task_cx_ptr = &inner.tasks[next].task_cx as *const TaskContext;
            drop(inner);
            // before this, we should drop local variables that must be dropped manually
            unsafe {
                __switch(current_task_cx_ptr, next_task_cx_ptr);
            }
            // go back to user mode
        } else {
            panic!("All applications completed!");
        }
    }

    // my code
    fn undapte_syscall_info(&self, syscall_id: usize) {
        let mut inner = self.inner.exclusive_access();
        let cur_task = inner.current_task;
        inner.tasks[cur_task].syscall_info[syscall_id] += 1;
    }
    
    fn get_task_info(&self) -> ([u32;MAX_SYSCALL_NUM], usize) {
        let inner = self.inner.exclusive_access();
        let cur_task = inner.current_task;
        (inner.tasks[cur_task].syscall_info, inner.tasks[cur_task].first_run)
    }
    // my code

    // my code
    fn mmap(&self, start: usize, len: usize, port: usize) -> isize {
        use crate::mm::{VirtPageNum, VirtAddr};
        use crate::config::PAGE_SIZE;
        if start & (PAGE_SIZE-1) != 0 { // 虚拟地址没有对齐
            return -1
        }

        let start_va = VirtAddr::from(start);
        // start+len == end vaddr; end vaddr + (PAGE_SIZE-1) / PAGE_SIZE = end vpn
        let end_va = VirtAddr::from(VirtPageNum::from((start+len + PAGE_SIZE-1) / PAGE_SIZE)); // 向上取整

        use crate::mm::MapPermission;
        let permission = match port {
            1 => MapPermission::U | MapPermission::R,
            2 => MapPermission::U | MapPermission::W,
            3 => MapPermission::U | MapPermission::R | MapPermission::W,
            4 => MapPermission::U | MapPermission::U | MapPermission::X,
            5 => MapPermission::U | MapPermission::U | MapPermission::X | MapPermission::R,
            6 => MapPermission::U | MapPermission::U | MapPermission::X | MapPermission::W,
            7 => MapPermission::U | MapPermission::U | MapPermission::X | MapPermission::W | MapPermission::R,
            _ => return -1, // 权限错误
        };

        let mut inner = TASK_MANAGER.inner.exclusive_access();
        let cur_task = inner.current_task;
        let cur_task_memory_set = &mut inner.tasks[cur_task].memory_set;
        
        let mut vpns: Vec<VirtPageNum> = Vec::new();
        for vpn in VirtPageNum::from(start_va).0..VirtPageNum::from(end_va).0 {
            vpns.push(VirtPageNum::from(vpn));
        }
        if cur_task_memory_set.pages_has_exit(vpns.clone()) {
            return -1; // 虚拟页已经被申请
        }
        cur_task_memory_set.insert_framed_area(start_va, end_va, permission);

        0
    }

    fn munmap(&self, start: usize, len: usize) -> isize {
        use crate::mm::{VirtPageNum, VirtAddr};
        use crate::config::PAGE_SIZE;
        if start & (PAGE_SIZE-1) != 0 { // 虚拟地址没有对齐
            return -1
        }

        let start_va = VirtAddr::from(start);
        // start+len == end vaddr; end vaddr + (PAGE_SIZE-1) / PAGE_SIZE = end vpn
        let end_va = VirtAddr::from(VirtPageNum::from((start+len + PAGE_SIZE-1) / PAGE_SIZE)); // 向上取整
        
        let mut inner = TASK_MANAGER.inner.exclusive_access();
        let cur_task = inner.current_task;
        let cur_task_memory_set = &mut inner.tasks[cur_task].memory_set;
        
        let mut vpns: Vec<VirtPageNum> = Vec::new();
        for vpn in VirtPageNum::from(start_va).0..VirtPageNum::from(end_va).0 {
            vpns.push(VirtPageNum::from(vpn));
        }
        if ! cur_task_memory_set.pages_all_exit(vpns.clone()) {
            return -1; // 虚拟页没有被申请
        }

        cur_task_memory_set.remove_pages(vpns.clone());

        0
    }
    // my code

}

// my code
/// run update task info
pub fn run_undapte_syscall_info(syscall_id: usize) {
    TASK_MANAGER.undapte_syscall_info(syscall_id);
}
/// run get syscall info
pub fn run_get_task_info() -> ([u32;MAX_SYSCALL_NUM], usize) {
    TASK_MANAGER.get_task_info()
}
/// run get page
pub fn run_mmap(start: usize, len: usize, port: usize) ->isize {
    TASK_MANAGER.mmap(start, len, port)
}
/// run munmap page
pub fn run_munmap(start: usize, len: usize) -> isize {
    TASK_MANAGER.munmap(start, len)
}
// my code

/// Run the first task in task list.
pub fn run_first_task() {
    TASK_MANAGER.run_first_task();
}

/// Switch current `Running` task to the task we have found,
/// or there is no `Ready` task and we can exit with all applications completed
fn run_next_task() {
    TASK_MANAGER.run_next_task();
}

/// Change the status of current `Running` task into `Ready`.
fn mark_current_suspended() {
    TASK_MANAGER.mark_current_suspended();
}

/// Change the status of current `Running` task into `Exited`.
fn mark_current_exited() {
    TASK_MANAGER.mark_current_exited();
}

/// Suspend the current 'Running' task and run the next task in task list.
pub fn suspend_current_and_run_next() {
    mark_current_suspended();
    run_next_task();
}

/// Exit the current 'Running' task and run the next task in task list.
pub fn exit_current_and_run_next() {
    mark_current_exited();
    run_next_task();
}

/// Get the current 'Running' task's token.
pub fn current_user_token() -> usize {
    TASK_MANAGER.get_current_token()
}

/// Get the current 'Running' task's trap contexts.
pub fn current_trap_cx() -> &'static mut TrapContext {
    TASK_MANAGER.get_current_trap_cx()
}

/// Change the current 'Running' task's program break
pub fn change_program_brk(size: i32) -> Option<usize> {
    TASK_MANAGER.change_current_program_brk(size)
}
