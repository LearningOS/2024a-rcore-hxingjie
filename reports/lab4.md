# Lab 4 实验报告

## 荣誉守则

1. 在完成本次实验的过程（含此前学习的过程）中，我曾分别与 **以下各位** 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：

   > 

2. 此外，我也参考了 **以下资料** ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：

   > *rCore-Camp-Guide-2024A 文档*

3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。



## 一 实现功能

1.linkat

(1) 为Inode实现link方法

```rust
pub fn link(&self, old_name: &str, new_name: &str)
```

只有root inode会调用此方法。

首先根据old_name获取对应的inode, 使其nlink字段加1。根据old_name获取对应的inode_id，修改root inode对应的disk inode，先使用increase_size增加大小，然后使用new name和inode_id构造得到新的DirEntry，将其写入root inode对应的disk node即可

(2) 在os/src/fs/inode.rs新增接口

```rust
pub fn link_file(old_name: &str, new_name: &str)
```

负责调用link方法

(3) 在sys_linkat中，首先使用translated_str将指针转换为String变量，再调用link_file接口即可。



2.unlinkat

(1) 为Inode实现unlink方法

```rust
pub fn unlink(&self, name: &str) -> isize
```

只有root inode会调用此方法。

首先根据name获取对应的inode, 使其nlink字段减1，并记录nlink是否已经为0。修改root inode对应的disk inode，遍历disk inode的DirEntry以找到name对应的DirEntry，将其置为空即可。判断是否nlink是否已经为0，如果为0，就调用inode的clear方法清空对应的空间

(2) 在os/src/fs/inode.rs新增接口

```rust
pub fn unlink_file(name: &str) -> isize
```

负责调用unlink方法

(3) 在sys_unlinkat中，首先使用translated_str将指针转换为String变量，再调用unlink_file接口即可。



3.fstat

(1) 为Inode实现find_inode_info方法

```rust
pub fn find_inode_info(&self, name: &str) -> (u32, u32, bool)
```

只有root inode会调用此方法。

首先根据name获取对应的inode_id，再根据name获取对应的inode，读取该inode对应的disk_inode的nlink字段，以及调用其is_file方法获取mode，将这些数据组合成元组返回。

(2) 在os/src/fs/inode.rs新增接口

```rust
pub fn get_inode_info(name: &str) -> (u32, u32, bool)
```

负责调用find_inode_info方法

(3) 修改TaskControlBlockInner中的fd_table类型为：

```rust
Vec<Option<(Arc<dyn File + Send + Sync>, String)>>，
```

(4) 为 Stat 实现方法

```rust
pub fn new(ino: u64, mode: StatMode, nlink: u32) -> Self
```

(5) 在sys_fstat中，获取当前任务的任务控制块的inner，根据fd取出fd_table对应的项，获取对应的name，使用name调用get_inode_info方法获取该文件的inode_id，nlink，mode。

使用的得到的数据构造Stat，使用translated_byte_buffer获取物理内存，写入Stat即可。



## 二 问答作业

1.在我们的easy-fs中，root inode起着什么作用？如果root inode中的内容损坏了，会发生什么？

因为简化了文件系统，所有的文件都在根目录下，所以 root inode 记录了该文件系统中的所有文件的 DirEntry ，是文件系统的管理者。

如果root inode中的内容损坏了，整个文件系统就混乱了，虽然文件内容没有影响，但是已经不能找到需要读写的文件了。



2.举出使用 pipe 的一个实际应用的例子。

```shell
cat filename | wc -l
```

使用管道将cat的输出作为wc的输入，实现文件的行数统计



3.如果需要在多个进程间互相通信，则需要为每一对进程建立一个管道，非常繁琐，请设计一个更易用的多进程通信机制。

共享内存

系统创建一个共享内存段，将共享内存段映射到调用进程的地址空间，之后进程就可以方便的读写这段空间。共享内存段可以映射给不同的进程，不同的进程读写同一块空间即可实现通信。
