use crate::sync::{Condvar, Mutex, MutexBlocking, MutexSpin, Semaphore};
use crate::task::{block_current_and_run_next, current_process, current_task};
use crate::timer::{add_timer, get_time_ms};
use alloc::sync::Arc;
use alloc::vec;

/// sleep syscall
pub fn sys_sleep(ms: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_sleep",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let expire_ms = get_time_ms() + ms;
    let task = current_task().unwrap();
    add_timer(expire_ms, task);
    block_current_and_run_next();
    0
}
/// mutex create syscall
pub fn sys_mutex_create(blocking: bool) -> isize {
    // return mutex_id
    trace!(
        "kernel:pid[{}] tid[{}] sys_mutex_create",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let mutex: Option<Arc<dyn Mutex>> = if !blocking { // 选择锁的类别
        Some(Arc::new(MutexSpin::new()))
    } else {
        Some(Arc::new(MutexBlocking::new()))
    };
    let mut process_inner = process.inner_exclusive_access();
    if let Some(id) = process_inner
        .mutex_list
        .iter()
        .enumerate()
        .find(|(_, item)| item.is_none())
        .map(|(id, _)| id)
    {
        process_inner.mutex_list[id] = mutex; // 有空元素

        // my code
        assert_eq!(process_inner.mutex_list.len(), process_inner.mutex_available.len());
        process_inner.mutex_available[id] = 1;
        // my code

        id as isize
    } else {
        assert_eq!(process_inner.mutex_list.len(), process_inner.mutex_available.len()); // my code
        
        process_inner.mutex_list.push(mutex);

        // my code
        process_inner.mutex_available.push(1);
        for i in process_inner.mutex_allocation.iter_mut() {
            i.push(0);
        }
        for i in process_inner.mutex_need.iter_mut() {
            i.push(0);
        }
        // my code

        process_inner.mutex_list.len() as isize - 1
    }
}
/// mutex lock syscall
pub fn sys_mutex_lock(mutex_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_mutex_lock",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    
    // my code
    let tid = current_task().unwrap().inner_exclusive_access().res.as_ref().expect("tcb's res is none").tid;
    process_inner.mutex_need[tid][mutex_id] += 1;
    
    if process_inner.enable_deadlock_detect == 1 { // 需要进行死锁检测
        let mutex_sz = process_inner.mutex_available.len();
        let thread_sz = process_inner.mutex_allocation.len();
        let mut finish_sz: usize = 0;

        let mut work = process_inner.mutex_available.clone();
        let mut finish = vec![false; thread_sz];

        // for i in 0..thread_sz {
        //     for j in 0..mutex_sz {
        //         if finish[i] == false && work[j] >= process_inner.mutex_need[i][j] {
        //             work[j] += process_inner.mutex_allocation[i][j];
        //             finish[i] = true;
        //             finish_sz += 1;
        //         }
        //     }
        // }

        let mut i = 0;
        while i < thread_sz {
            if finish[i] == false {
                let mut more_than = true;
                for j in 0..mutex_sz {
                    if process_inner.mutex_need[i][j] > 0 
                        && work[j] < process_inner.mutex_need[i][j] { // 有一类不满足
                        more_than = false;
                        break;
                    }
               }
               if more_than {
                    for j in 0..mutex_sz {
                        work[j] += process_inner.mutex_allocation[i][j];
                    }
                    finish[i] = true;
                    finish_sz += 1;
                    i = 0;
                    continue;
               }
            }
            i += 1;
        }

        if finish_sz < thread_sz {
            process_inner.mutex_need[tid][mutex_id] -= 1; // 恢复
            return -0xdead;
        }
    }
    // my code

    let mutex = Arc::clone(process_inner.mutex_list[mutex_id].as_ref().unwrap());

    drop(process_inner);
    drop(process);

    mutex.blocking_lock(mutex_id); // add mutex_id
    0
}
/// mutex unlock syscall
pub fn sys_mutex_unlock(mutex_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_mutex_unlock",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let mutex = Arc::clone(process_inner.mutex_list[mutex_id].as_ref().unwrap());
    drop(process_inner);
    drop(process);
    mutex.blocking_unlock(mutex_id);
    0
}
/// semaphore create syscall
pub fn sys_semaphore_create(res_count: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_semaphore_create",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let id = if let Some(id) = process_inner
        .semaphore_list
        .iter()
        .enumerate()
        .find(|(_, item)| item.is_none())
        .map(|(id, _)| id)
    {
        process_inner.semaphore_list[id] = Some(Arc::new(Semaphore::new(res_count)));

        // my code
        assert_eq!(process_inner.semaphore_list.len(), process_inner.semaphore_available.len());
        process_inner.semaphore_available[id] = res_count as isize;
        // my code

        id
    } else {
        process_inner
            .semaphore_list
            .push(Some(Arc::new(Semaphore::new(res_count))));

        // my code
        process_inner.semaphore_available.push(res_count as isize);
        for i in process_inner.semaphore_allocation.iter_mut() {
            i.push(0);
        }
        for i in process_inner.semaphore_need.iter_mut() {
            i.push(0);
        }
        // my code

        process_inner.semaphore_list.len() - 1
    };
    id as isize
}
/// semaphore up syscall
pub fn sys_semaphore_up(sem_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_semaphore_up",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let sem = Arc::clone(process_inner.semaphore_list[sem_id].as_ref().unwrap());
    drop(process_inner);
    sem.up(sem_id);
    0
}
/// semaphore down syscall
pub fn sys_semaphore_down(sem_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_semaphore_down",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    
    // my code
    let tid = current_task().unwrap().inner_exclusive_access().res.as_ref().expect("tcb's res is none").tid;
    process_inner.semaphore_need[tid][sem_id] += 1;
    
    if process_inner.enable_deadlock_detect == 1 { // 需要进行死锁检测
        let semaphore_sz = process_inner.semaphore_available.len();
        let thread_sz = process_inner.semaphore_allocation.len();
        let mut finish_sz: usize = 0;

        let mut work = process_inner.semaphore_available.clone();
        let mut finish = vec![false; thread_sz];

        let mut i = 0;
        while i < thread_sz {
            if finish[i] == false {
                let mut more_than = true;
                for j in 0..semaphore_sz {
                    if process_inner.semaphore_need[i][j] > 0 
                        && work[j] < process_inner.semaphore_need[i][j] { // 有一类不满足
                        more_than = false;
                        break;
                    }
               }
               if more_than {
                    for j in 0..semaphore_sz {
                        work[j] += process_inner.semaphore_allocation[i][j];
                    }
                    finish[i] = true;
                    finish_sz += 1;
                    i = 0;
                    continue;
               }
            }
            i += 1;
        }

        if finish_sz < thread_sz {
            process_inner.semaphore_need[tid][sem_id] -= 1; // 恢复
            return -0xdead;
        }
    }
    // my code

    let sem = Arc::clone(process_inner.semaphore_list[sem_id].as_ref().unwrap());
    drop(process_inner);
    sem.down(sem_id);
    0
}
/// condvar create syscall
pub fn sys_condvar_create() -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_condvar_create",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let id = if let Some(id) = process_inner
        .condvar_list
        .iter()
        .enumerate()
        .find(|(_, item)| item.is_none())
        .map(|(id, _)| id)
    {
        process_inner.condvar_list[id] = Some(Arc::new(Condvar::new()));
        id
    } else {
        process_inner
            .condvar_list
            .push(Some(Arc::new(Condvar::new())));
        process_inner.condvar_list.len() - 1
    };
    id as isize
}
/// condvar signal syscall
pub fn sys_condvar_signal(condvar_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_condvar_signal",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let condvar = Arc::clone(process_inner.condvar_list[condvar_id].as_ref().unwrap());
    drop(process_inner);
    condvar.signal();
    0
}
/// condvar wait syscall
pub fn sys_condvar_wait(condvar_id: usize, mutex_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_condvar_wait",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let condvar = Arc::clone(process_inner.condvar_list[condvar_id].as_ref().unwrap());
    let mutex = Arc::clone(process_inner.mutex_list[mutex_id].as_ref().unwrap());
    drop(process_inner);
    condvar.wait(mutex);
    0
}
/// enable deadlock detection syscall
///
/// YOUR JOB: Implement deadlock detection, but might not all in this syscall
pub fn sys_enable_deadlock_detect(enabled: usize) -> isize {
    // trace!("kernel: sys_enable_deadlock_detect NOT IMPLEMENTED");
    // -1
    // 为当前进程启用或禁用 deadlock_detect
    if enabled != 0 && enabled != 1 {
        return -1;
    }
    //if can_open() {
    //    return -1;
    //}
    // 获取 PCB
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    process_inner.enable_deadlock_detect = enabled;

    0
}
