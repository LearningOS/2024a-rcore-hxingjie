use crate::{
    mm::kernel_token,
    task::{add_task, current_task, TaskControlBlock},
    trap::{trap_handler, TrapContext},
};
use alloc::sync::Arc;
use alloc::vec; // my code

/// thread create syscall
pub fn sys_thread_create(entry: usize, arg: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_thread_create",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let task = current_task().unwrap(); // 找到当前执行的线程
    let process = task.process.upgrade().unwrap(); // 找到该线程所属的进程
    // create a new thread
    // 建立与所属进程的所属关系，分配线程的用户态栈、内核态栈、跳板页
    let new_task = Arc::new(TaskControlBlock::new(
        Arc::clone(&process),
        task.inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .ustack_base,
        true,
    ));
    // add new task to scheduler 加入调度队列
    add_task(Arc::clone(&new_task));
    let new_task_inner = new_task.inner_exclusive_access();
    let new_task_res = new_task_inner.res.as_ref().unwrap();
    let new_task_tid = new_task_res.tid;
    let mut process_inner = process.inner_exclusive_access();
    // add new thread to current process 将新线程放入进程的线程列表中
    let tasks = &mut process_inner.tasks;
    while tasks.len() < new_task_tid + 1 {
        tasks.push(None);
    }
    tasks[new_task_tid] = Some(Arc::clone(&new_task));
    // 初始化新线程在用户态地址空间中的Trap上下文
    // 设置线程的函数入口点和用户栈，使得第一次进入用户态时能从线程起始位置开始正确执行
    // 设置内核栈和陷入函数指针trap_handler，使得Trap的时候用户态的线程能正确进入内核态
    let new_task_trap_cx = new_task_inner.get_trap_cx();
    *new_task_trap_cx = TrapContext::app_init_context(
        entry,
        new_task_res.ustack_top(),
        kernel_token(),
        new_task.kstack.get_top(),
        trap_handler as usize,
    );
    (*new_task_trap_cx).x[10] = arg;

    // my code 创建新线程，维护 mutex_allocation
    let row = process_inner.mutex_allocation.len();
    if new_task_tid == row {
        let mutex_sz = process_inner.mutex_allocation[0].len();
        process_inner.mutex_allocation.push(vec![0; mutex_sz]);
        process_inner.mutex_need.push(vec![0; mutex_sz]);
    } else if new_task_tid < row {
        let mutex_sz = process_inner.mutex_allocation[0].len();
        process_inner.mutex_allocation[new_task_tid] = vec![0; mutex_sz];
        process_inner.mutex_need[new_task_tid] = vec![0; mutex_sz];
    } else {
        panic!("os/src/syscall/thread.rs error!!");
    }
    // my code

    // my code 创建新线程，维护 semaphore_allocation
    let row = process_inner.semaphore_allocation.len();
    if new_task_tid == row {
        let semaphore_sz = process_inner.semaphore_allocation[0].len();
        process_inner.semaphore_allocation.push(vec![0; semaphore_sz]);
        process_inner.semaphore_need.push(vec![0; semaphore_sz]);
    } else if new_task_tid < row {
        let semaphore_sz = process_inner.semaphore_allocation[0].len();
        process_inner.semaphore_allocation[new_task_tid] = vec![0; semaphore_sz];
        process_inner.semaphore_need[new_task_tid] = vec![0; semaphore_sz];
    } else {
        panic!("os/src/syscall/thread.rs error!!");
    }
    // my code

    new_task_tid as isize
}
/// get current thread id syscall
pub fn sys_gettid() -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_gettid",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    current_task()
        .unwrap()
        .inner_exclusive_access()
        .res
        .as_ref()
        .unwrap()
        .tid as isize
}

/// wait for a thread to exit syscall
///
/// thread does not exist, return -1
/// thread has not exited yet, return -2
/// otherwise, return thread's exit code
pub fn sys_waittid(tid: usize) -> i32 {
    trace!(
        "kernel:pid[{}] tid[{}] sys_waittid",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let task = current_task().unwrap();
    let process = task.process.upgrade().unwrap();
    let task_inner = task.inner_exclusive_access();
    let mut process_inner = process.inner_exclusive_access();
    // a thread cannot wait for itself
    if task_inner.res.as_ref().unwrap().tid == tid {
        return -1;
    }
    let mut exit_code: Option<i32> = None;
    let waited_task = process_inner.tasks[tid].as_ref();
    if let Some(waited_task) = waited_task {
        if let Some(waited_exit_code) = waited_task.inner_exclusive_access().exit_code {
            exit_code = Some(waited_exit_code);
        }
    } else {
        // waited thread does not exist
        return -1;
    }
    if let Some(exit_code) = exit_code {
        // dealloc the exited thread
        process_inner.tasks[tid] = None;
        exit_code
    } else {
        // waited thread has not exited
        -2
    }
}
