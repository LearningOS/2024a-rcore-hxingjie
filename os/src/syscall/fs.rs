//! File and filesystem-related syscalls
use crate::fs::{get_inode_info, link_file, open_file, unlink_file, OpenFlags, Stat, StatMode};
use crate::mm::{translated_byte_buffer, translated_str, UserBuffer, vaddr_to_paddr};
use crate::task::{current_task, current_user_token};

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    trace!("kernel:pid[{}] sys_write", current_task().unwrap().pid.0);
    let token = current_user_token(); // for get page table
    let task = current_task().unwrap(); // get tcb(arc's clone)
    let inner = task.inner_exclusive_access(); // get tcb inner
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        // clone file.0
        let file = file.0.clone(); // arc's clone
        if !file.writable() { // can't write
            return -1;
        }
        // release current task TCB manually to avoid multi-borrow
        drop(inner);
        // 获取物理内存的可变引用
        file.write(UserBuffer::new(translated_byte_buffer(token, buf, len))) as isize
    } else {
        -1
    }
}

pub fn sys_read(fd: usize, buf: *const u8, len: usize) -> isize {
    trace!("kernel:pid[{}] sys_read", current_task().unwrap().pid.0);
    let token = current_user_token();
    let task = current_task().unwrap();
    let inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        let file = file.0.clone();
        if !file.readable() {
            return -1;
        }
        // release current task TCB manually to avoid multi-borrow
        drop(inner);
        trace!("kernel: sys_read .. file.read");
        file.read(UserBuffer::new(translated_byte_buffer(token, buf, len))) as isize
    } else {
        -1
    }
}

pub fn sys_open(path: *const u8, flags: u32) -> isize {
    trace!("kernel:pid[{}] sys_open", current_task().unwrap().pid.0);
    let task = current_task().unwrap();
    let token = current_user_token();
    let path = translated_str(token, path);
    if let Some(inode) = open_file(path.as_str(), OpenFlags::from_bits(flags).unwrap()) {
        let mut inner = task.inner_exclusive_access();
        let fd = inner.alloc_fd();
        inner.fd_table[fd] = Some((inode, path));
        fd as isize
    } else {
        -1
    }
}

pub fn sys_close(fd: usize) -> isize {
    trace!("kernel:pid[{}] sys_close", current_task().unwrap().pid.0);
    let task = current_task().unwrap();
    let mut inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if inner.fd_table[fd].is_none() {
        return -1;
    }
    inner.fd_table[fd].take();
    0
}

/// YOUR JOB: Implement fstat.
pub fn sys_fstat(fd: usize, st: *mut Stat) -> isize {
    // trace!(
    //     "kernel:pid[{}] sys_fstat NOT IMPLEMENTED",
    //     current_task().unwrap().pid.0
    // );
    // -1
    let task = current_task().unwrap();
    let inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }

    if let Some(file) = &inner.fd_table[fd] {
        let _f = file.0.clone();
        let name = file.1.clone();
        drop(inner); // 结束借用

        let (ino, nlink, isfile) = get_inode_info(name.as_str());
        unsafe {
            let mut paddr = vaddr_to_paddr(st as usize);
            (*(paddr as *mut u64)) = 0; // write dev
            paddr += 8;

            (*(paddr as *mut u64)) = ino as u64; // write ino
            paddr += 8;

            (*(paddr as *mut StatMode)) = if isfile { StatMode::FILE } else { StatMode::DIR }; // write mode
            paddr += 4;

            (*(paddr as *mut u32)) = nlink; // write nlink
        }
        0
    } else {
        -1
    }
}

/// YOUR JOB: Implement linkat.
pub fn sys_linkat(old_name: *const u8, new_name: *const u8) -> isize {
    // trace!(
    //     "kernel:pid[{}] sys_linkat NOT IMPLEMENTED",
    //     current_task().unwrap().pid.0
    // );
    // -1
    if old_name == new_name { // 相同文件
        return -1;
    }
    let token = current_user_token();
    let old_path = translated_str(token, old_name);
    let new_path = translated_str(token, new_name);
    link_file(old_path.as_str(), new_path.as_str());

    0
}

/// YOUR JOB: Implement unlinkat.
pub fn sys_unlinkat(name: *const u8) -> isize {
    // trace!(
    //     "kernel:pid[{}] sys_unlinkat NOT IMPLEMENTED",
    //     current_task().unwrap().pid.0
    // );
    // -1
    let token = current_user_token();
    let path = translated_str(token, name);
    unlink_file(path.as_str())

}
