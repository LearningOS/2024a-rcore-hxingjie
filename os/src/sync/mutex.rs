//! Mutex (spin-like and blocking(sleep))

use super::UPSafeCell;
use crate::task::TaskControlBlock;
use crate::task::{block_current_and_run_next, suspend_current_and_run_next};
use crate::task::{current_task, wakeup_task};
use alloc::{collections::VecDeque, sync::Arc};

// my code
use crate::task::current_process;
// my code

/// Mutex trait
pub trait Mutex: Sync + Send {
    /// Lock the mutex
    fn lock(&self);
    /// Unlock the mutex
    fn unlock(&self);

    // my code
    ///
    fn blocking_lock(&self, _mutex_id: usize) {
        
    }
    ///
    fn blocking_unlock(&self, _mutex_id: usize) {
        
    }
    // my code
}

/// Spinlock Mutex struct
pub struct MutexSpin {
    locked: UPSafeCell<bool>,
}

impl MutexSpin {
    /// Create a new spinlock mutex
    pub fn new() -> Self {
        Self {
            locked: unsafe { UPSafeCell::new(false) },
        }
    }
}

impl Mutex for MutexSpin {
    /// Lock the spinlock mutex
    fn lock(&self) {
        trace!("kernel: MutexSpin::lock");
        loop {
            let mut locked = self.locked.exclusive_access();
            if *locked {
                drop(locked);
                suspend_current_and_run_next();
                continue;
            } else {
                *locked = true;
                return;
            }
        }
    }

    fn unlock(&self) {
        trace!("kernel: MutexSpin::unlock");
        let mut locked = self.locked.exclusive_access();
        *locked = false;
    }

    /// Lock the spinlock mutex
    fn blocking_lock(&self, _mutex_id: usize) {
        trace!("kernel: MutexSpin::lock");
        loop {
            let mut locked = self.locked.exclusive_access();
            if *locked {
                drop(locked);
                suspend_current_and_run_next();
                continue;
            } else {
                *locked = true;
                return;
            }
        }
    }

    fn blocking_unlock(&self, _mutex_id: usize) {
        trace!("kernel: MutexSpin::unlock");
        let mut locked = self.locked.exclusive_access();
        *locked = false;
    }
}

/// Blocking Mutex struct
pub struct MutexBlocking {
    inner: UPSafeCell<MutexBlockingInner>,
}

pub struct MutexBlockingInner {
    locked: bool,
    wait_queue: VecDeque<Arc<TaskControlBlock>>,
}

impl MutexBlocking {
    /// Create a new blocking mutex
    pub fn new() -> Self {
        trace!("kernel: MutexBlocking::new");
        Self {
            inner: unsafe {
                UPSafeCell::new(MutexBlockingInner {
                    locked: false,
                    wait_queue: VecDeque::new(),
                })
            },
        }
    }
}

impl Mutex for MutexBlocking {
    /// lock the blocking mutex
    fn lock(&self) {
        trace!("kernel: MutexBlocking::lock");
        let mut mutex_inner = self.inner.exclusive_access();
        if mutex_inner.locked {
            mutex_inner.wait_queue.push_back(current_task().unwrap()); // 已经上锁，加入等待队列
            drop(mutex_inner);
            block_current_and_run_next(); // 阻塞当前任务
        } else {
            mutex_inner.locked = true;
        }
    }

    /// unlock the blocking mutex
    fn unlock(&self) {
        trace!("kernel: MutexBlocking::unlock");
        let mut mutex_inner = self.inner.exclusive_access();
        assert!(mutex_inner.locked);
        if let Some(waking_task) = mutex_inner.wait_queue.pop_front() {
            wakeup_task(waking_task);
        } else {
            mutex_inner.locked = false;
        }
    }

    fn blocking_lock(&self, _mutex_id: usize) {
        trace!("kernel: MutexBlocking::lock");
        let mut mutex_inner = self.inner.exclusive_access();
        if mutex_inner.locked {
            mutex_inner.wait_queue.push_back(current_task().unwrap()); // 已经上锁，加入等待队列
            drop(mutex_inner);
            block_current_and_run_next(); // 阻塞当前任务
            
            // my code
            let process = current_process();
            let mut process_inner = process.inner_exclusive_access();        
            let tid = current_task().unwrap().inner_exclusive_access().res.as_ref().expect("tcb's res is none").tid;
            process_inner.mutex_allocation[tid][_mutex_id] += 1;
            process_inner.mutex_need[tid][_mutex_id] -= 1;
            // my code
        } else {
            mutex_inner.locked = true;

            // my code
            let process = current_process();
            let mut process_inner = process.inner_exclusive_access();        
            let tid = current_task().unwrap().inner_exclusive_access().res.as_ref().expect("tcb's res is none").tid;
            process_inner.mutex_available[_mutex_id] -= 1;
            process_inner.mutex_allocation[tid][_mutex_id] += 1;
            process_inner.mutex_need[tid][_mutex_id] -= 1;
            // my code
        }
    }

    fn blocking_unlock(&self, _mutex_id: usize) {
        trace!("kernel: MutexBlocking::unlock");
        let mut mutex_inner = self.inner.exclusive_access();
        assert!(mutex_inner.locked);

        // my code
        let process = current_process();
        let mut process_inner = process.inner_exclusive_access();        
        let tid = current_task().unwrap().inner_exclusive_access().res.as_ref().expect("tcb's res is none").tid;
        // my code

        if let Some(waking_task) = mutex_inner.wait_queue.pop_front() {
            // my code
            process_inner.mutex_allocation[tid][_mutex_id] -= 1;
            
            drop(process_inner);
            drop(process);
            // my code

            wakeup_task(waking_task);
        } else {
            mutex_inner.locked = false;

            // my code
            process_inner.mutex_available[_mutex_id] += 1;
            process_inner.mutex_allocation[tid][_mutex_id] -= 1;
            // my code
        }
    }
}
