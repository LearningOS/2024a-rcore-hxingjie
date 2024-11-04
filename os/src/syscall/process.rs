//! Process management syscalls
use crate::{
    config::MAX_SYSCALL_NUM, task::{
        change_program_brk, exit_current_and_run_next, suspend_current_and_run_next, TaskStatus,
    }
};

/// 
#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    ///
    pub sec: usize,
    ///
    pub usec: usize,
}

/// Task information
#[allow(dead_code)]
pub struct TaskInfo {
    /// Task status in it's life cycle
    status: TaskStatus, // 1
    /// The numbers of syscall called by task
    syscall_times: [u32; MAX_SYSCALL_NUM], // 2000
    /// Total running time of task
    time: usize, // 8
}

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    // my code

    use crate::task::current_user_token;
    use crate::mm::translated_byte_buffer;
    use crate::timer::get_time_us;

    let us = get_time_us();
    let time_val = TimeVal {
        sec: us / 1_000_000,
        usec: us % 1_000_000,
    };

    let mut src = &time_val as *const TimeVal as *const u8;
    let dest = _ts as *const u8;

    let buffers = translated_byte_buffer(
        current_user_token(), dest, core::mem::size_of::<TimeVal>());

    for buffer in buffers {
        unsafe {
            buffer.copy_from_slice(
                core::slice::from_raw_parts(src, buffer.len()));
            src = src.add(buffer.len());
        }
    }

    0
    // my code
    
    //-1
}

/// YOUR JOB: Finish sys_task_info to pass testcases
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TaskInfo`] is splitted by two pages ?
pub fn sys_task_info(_ti: *mut TaskInfo) -> isize {
    trace!("kernel: sys_task_info NOT IMPLEMENTED YET!");

    use crate::task::current_user_token;
    use crate::mm::translated_byte_buffer;
    use crate::task::run_get_task_info;
    use crate::timer::get_time_ms;

    let (syscall_info, first_run) = run_get_task_info();

    let task_info = TaskInfo {
        status: TaskStatus::Running,
        syscall_times: syscall_info,
        time: get_time_ms() - first_run,
    };
    let mut src = &task_info as *const TaskInfo as *const u8;
    let dest = _ti as *const u8;

    let buffers = translated_byte_buffer(
        current_user_token(), dest, core::mem::size_of::<TaskInfo>());
        
    for buffer in buffers {
        unsafe {
            buffer.copy_from_slice(
                core::slice::from_raw_parts(src, buffer.len()));
            src = src.add(buffer.len());
        }
    }

    0
    //-1
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(start: usize, len: usize, port: usize) -> isize {
    trace!("kernel: sys_mmap NOT IMPLEMENTED YET!");
    use crate::task::run_mmap;
    run_mmap(start, len, port) 
    //-1
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(start: usize, len: usize) -> isize {
    trace!("kernel: sys_munmap NOT IMPLEMENTED YET!");
    use crate::task::run_munmap;
    run_munmap(start, len)
    //-1
}
/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
