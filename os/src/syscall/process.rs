//! Process management syscalls
use crate::{
    config::MAX_SYSCALL_NUM, mm::vaddr_to_paddr, task::{
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
    use crate::timer::get_time_us;
    let us = get_time_us();

    use crate::mm::vaddr_to_paddr;
    let paddr = vaddr_to_paddr(_ts as usize);
    let p = paddr as *mut usize;
    unsafe { *p = us / 1_000_000; }

    let paddr = vaddr_to_paddr((_ts as usize) + 8);
    let p = paddr as *mut usize;
    unsafe { *p = us % 1_000_000; }
    0
    // my code
    
    //-1
}

/// YOUR JOB: Finish sys_task_info to pass testcases
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TaskInfo`] is splitted by two pages ?
pub fn sys_task_info(_ti: *mut TaskInfo) -> isize {
    trace!("kernel: sys_task_info NOT IMPLEMENTED YET!");

    use crate::task::run_get_task_info;
    use crate::timer::get_time_ms;
    let (syscall_info, first_run) = run_get_task_info();

    // syscall_times 2000
    // time 8
    // status 1
    let mut ptr = _ti as usize;
    unsafe {
        for idx in 0..MAX_SYSCALL_NUM {
            let paddr = vaddr_to_paddr(ptr);
            (*(paddr as *mut u32)) = syscall_info[idx];

            ptr += 4;
        }
        let paddr = vaddr_to_paddr(ptr);
        (*(paddr as *mut usize)) = get_time_ms() - first_run;

        ptr += 8;
        let paddr = vaddr_to_paddr(ptr);
        (*(paddr as *mut TaskStatus)) = TaskStatus::Running;
    }

    // let (_, paddr) = vaddr_to_paddr(_ti as usize);
    // let _ti = paddr.0 as *mut TaskInfo;

    // unsafe {
    //     (*_ti).status = TaskStatus::Running;
    //     (*_ti).syscall_times = syscall_info;
    //     (*_ti).time = get_time_ms() - first_run;
    // }
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
