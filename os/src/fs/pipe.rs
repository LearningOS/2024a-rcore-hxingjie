use super::File;
use crate::mm::UserBuffer;
use crate::sync::UPSafeCell;
use alloc::sync::{Arc, Weak};

use crate::task::suspend_current_and_run_next;

/// IPC pipe
pub struct Pipe {
    readable: bool,
    writable: bool,
    buffer: Arc<UPSafeCell<PipeRingBuffer>>,
}

impl Pipe {
    /// create readable pipe
    pub fn read_end_with_buffer(buffer: Arc<UPSafeCell<PipeRingBuffer>>) -> Self {
        Self {
            readable: true,
            writable: false,
            buffer,
        }
    }
    /// create writable pipe
    pub fn write_end_with_buffer(buffer: Arc<UPSafeCell<PipeRingBuffer>>) -> Self {
        Self {
            readable: false,
            writable: true,
            buffer,
        }
    }
}

const RING_BUFFER_SIZE: usize = 32;

#[derive(Copy, Clone, PartialEq)]
enum RingBufferStatus {
    Full,
    Empty,
    Normal,
}

pub struct PipeRingBuffer {
    arr: [u8; RING_BUFFER_SIZE],
    head: usize,
    tail: usize,
    status: RingBufferStatus,
    write_end: Option<Weak<Pipe>>,
}

impl PipeRingBuffer {
    pub fn new() -> Self {
        Self {
            arr: [0; RING_BUFFER_SIZE],
            head: 0,
            tail: 0,
            status: RingBufferStatus::Empty,
            write_end: None,
        }
    }
    pub fn set_write_end(&mut self, write_end: &Arc<Pipe>) {
        self.write_end = Some(Arc::downgrade(write_end));
    }
    pub fn write_byte(&mut self, byte: u8) {
        self.status = RingBufferStatus::Normal;
        self.arr[self.tail] = byte;
        self.tail = (self.tail + 1) % RING_BUFFER_SIZE;
        if self.tail == self.head {
            self.status = RingBufferStatus::Full;
        }
    }
    pub fn read_byte(&mut self) -> u8 {
        self.status = RingBufferStatus::Normal; // set status
        let c = self.arr[self.head]; // read one byte
        self.head = (self.head + 1) % RING_BUFFER_SIZE; // ring queue
        if self.head == self.tail { // after read one byte, head == tail, queue must be empty
            self.status = RingBufferStatus::Empty;
        }
        c // return one byte
    }
    pub fn available_read(&self) -> usize {
        if self.status == RingBufferStatus::Empty { // 根据 status 判断是否可读
            0
        } else if self.tail > self.head { // 返回可读的字节数
            self.tail - self.head
        } else { // 返回可读的字节数
            self.tail + RING_BUFFER_SIZE - self.head
        }
    }
    pub fn available_write(&self) -> usize {
        if self.status == RingBufferStatus::Full {
            0
        } else {
            RING_BUFFER_SIZE - self.available_read()
        }
    }
    pub fn all_write_ends_closed(&self) -> bool {
        self.write_end
            .as_ref()
            .unwrap()
            .upgrade()
            .is_none()
    }
}

/// Return (read_end, write_end)
pub fn make_pipe() -> (Arc<Pipe>, Arc<Pipe>) {
    let buffer = Arc::new(unsafe { UPSafeCell::new(PipeRingBuffer::new()) });

    let read_end = Arc::new(Pipe::read_end_with_buffer(buffer.clone()));
    let write_end = Arc::new(Pipe::write_end_with_buffer(buffer.clone()));
    buffer.exclusive_access().set_write_end(&write_end);

    (read_end, write_end)
}

impl File for Pipe {
    fn readable(&self) -> bool {
        self.readable
    }
    fn writable(&self) -> bool {
        self.writable
    }
    fn read(&self, buf: UserBuffer) -> usize {
        // buf 是物理空间的引用
        assert!(self.readable());
        let want_to_read = buf.len(); // 希望读取的字符数
        // `buf_iter` 将传入的应用缓冲区 `buf` 转化为一个能够逐字节对于缓冲区进行访问的迭代器，
      	// 每次调用 `buf_iter.next()` 即可按顺序取出用于访问缓冲区中一个字节的裸指针。
        let mut buf_iter = buf.into_iter(); // buff的迭代器
        let mut already_read = 0usize; // 已经从缓冲区读取的字符数
        loop {
            let mut ring_buffer = self.buffer.exclusive_access(); // 借用缓冲区
            let loop_read = ring_buffer.available_read(); // 当前缓冲区可读取的字符数
            if loop_read == 0 { // 当前缓冲区没有可读取的字符
                if ring_buffer.all_write_ends_closed() { // 当前缓冲区的写端是否都关闭
                    return already_read;
                }
                //在调用之前我们需要手动释放管道自身的锁，因为切换任务时候的 `__switch` 并不是一个正常的函数调用。
                drop(ring_buffer); // 归还缓冲区的引用
                suspend_current_and_run_next(); // 阻塞
                continue; // 唤醒后重新进入循环
            }
            for _ in 0..loop_read { // 读取缓冲区的全部字符
                if let Some(byte_ref) = buf_iter.next() {
                    unsafe {
                        *byte_ref = ring_buffer.read_byte(); // 写入字符
                    }
                    already_read += 1;
                    if already_read == want_to_read {
                        return want_to_read;
                    }
                } else {
                    return already_read; // buf_iter.next() == None
                }
            }
        }
    }
    fn write(&self, buf: UserBuffer) -> usize {
        assert!(self.writable());
        let want_to_write = buf.len();
        let mut buf_iter = buf.into_iter();
        let mut already_write = 0usize;
        loop {
            let mut ring_buffer = self.buffer.exclusive_access();
            let loop_write = ring_buffer.available_write();
            if loop_write == 0 {
                drop(ring_buffer);
                suspend_current_and_run_next();
                continue;
            }
            // write at most loop_write bytes
            for _ in 0..loop_write {
                if let Some(byte_ref) = buf_iter.next() {
                    ring_buffer.write_byte(unsafe { *byte_ref }); // 写入字符到缓冲区
                    already_write += 1;
                    if already_write == want_to_write {
                        return want_to_write;
                    }
                } else {
                    return already_write;
                }
            }
        }
    }
}
