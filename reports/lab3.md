# Lab 1 实验报告

## 荣誉守则

1. 在完成本次实验的过程（含此前学习的过程）中，我曾分别与 **以下各位** 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：

   > 

2. 此外，我也参考了 **以下资料** ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：

   > *rCore-Camp-Guide-2024A 文档*

3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。



## 一 实现功能

1.sys_spawn

在 `TaskControlBlock` 中添加 `creare_new_child` 方法:

```rust
pub fn spawn(self: &Arc<Self>, data: &[u8]) -> Arc<Self>
```

具体逻辑：

(1) 使用 `MemorySet::from_elf` 函数处理elf文件，得到 `memory_set`, `user_sp`, `entry_point`

(2) 获取 `trap_cx_ppn`，`pid_handle`，`kernel_stack`

(3) 构造 `TaskControlBlock`，注意添加 `parent`

(4) 更新 `parent.children`

(5) 准备 `TrapContext`



实现sys_spawn

首先获得当前任务的 `token`，使用 `translated_str` 查找文件名字，使用`get_app_data_by_name` 接口读取文件，获取当前任务的任务控制块，调用 `spawn` 创建子进程，将子进程添加进调度队列，返回 `pid`



2.stride 调度算法

(1) 在 TaskControlBlockInner 添加字段

```rust
pub struct TaskControlBlockInner {
		//..
    pub prio: u64,
    pub stride: u64,
}
```

在TaskControlBlock的new, fort, spawn中，将新进程的prio字段初始化为16，stride字段初始化为0。

(2) 在config.rs中添加BIG_STRIDE

```rust
pub const BIG_STRIDE: u64 = u64::MAX / 2;
```

(3) 在TaskManeger的fetch方法中添加逻辑

遍历ready_queue，找到stride最小的任务，更新该任务的stride字段，返回该任务

处理溢出:

如果abs(stride - min_stride) > BIG_STRIDE，较小值应为其中较大的值

更新时取模，stride %= BIG_STRIDE+1



## 二 问答作业

stride 算法深入

> stride 算法原理非常简单，但是有一个比较大的问题。例如两个 pass = 10 的进程，使用 8bit 无符号整形储存 stride， p1.stride = 255, p2.stride = 250，在 p2 执行一个时间片后，理论上下一次应该 p1 执行。
>
> - 实际情况是轮到 p1 执行吗？为什么？
>
> 不是，实际情况应该是轮到 p2 执行，因为 (p2.stride + 10) % 256 = 4，此时 p1.stride = 255，p2.stride = 4，此时还是 p2.stride 最小。
>
> 我们之前要求进程优先级 >= 2 其实就是为了解决这个问题。可以证明， **在不考虑溢出的情况下** , 在进程优先级全部 >= 2 的情况下，如果严格按照算法执行，那么 STRIDE_MAX – STRIDE_MIN <= BigStride / 2。
>
> - 为什么？尝试简单说明（不要求严格证明）。
>
> BigStride = 255
>
> 假设 p1.priority == 255, p2.priority == 2,
>
> 那么 p1.pass == 1, p2.pass == BigStride/2
>
> 
>
> 开始时 p1.stride = 0, p2.stride = 0
>
> (1)此时假设选中 p2, p2.stride += BigStride/p2.priority, p2.stride = BigStride/2
>
> 此时 STRIDE_MAX – STRIDE_MIN == BigStride/2 - 0 == BigStride / 2
>
> 之后一直会选择 p1,
>
> STRIDE_MAX – STRIDE_MIN == BigStride/2 - p1.stride < BigStride/2
>
> 直到 p1.stride == BigStride/2
>
> 如果(1)时选择p1, p1.stride += 1, p1.stride == 1, 下一次会选中 p2, p2.stride += BigStride/2, p2.stride == BigStride/2
>
> STRIDE_MAX – STRIDE_MIN == BigStride/2 - 1 < BigStride/2
>
> 
>
> (2)此时假设选中p2, p2.stride += BigStride/2, p2.stride == BigStride/2+BigStride/2 == 254
>
> 此时 STRIDE_MAX – STRIDE_MIN == (BigStride/2+BigStride/2) - BigStride/2+BigStride/2 == BigStride / 2
>
> 之后一直会选择 p1, 
>
> STRIDE_MAX – STRIDE_MIN == (BigStride/2+BigStride/2) - p1.stride < BigStride/2
>
> 直到 p1.stride == (BigStride/2+BigStride/2)
>
> 如果(2)时选择p1, p1.stride += 1, p1.stride == BigStride/2+1, 下一次会选中 p2, p2.stride += BigStride/2, p2.stride == BigStride/2+BigStride/2
>
> STRIDE_MAX – STRIDE_MIN == (BigStride/2+BigStride/2) - (BigStride/2+1) < BigStride/2
>
> 
>
> 所以 STRIDE_MAX – STRIDE_MIN <= BigStride / 2

> 
>
> - 已知以上结论，**考虑溢出的情况下**，可以为 Stride 设计特别的比较器，让 BinaryHeap<Stride> 的 pop 方法能返回真正最小的 Stride。补全下列代码中的 `partial_cmp` 函数，假设两个 Stride 永远不会相等。

```rust
use core::cmp::Ordering;

struct Stride(u64);

impl PartialOrd for Stride {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let BigStride: u64 = 255;
        if self.0 > other.0 && self.0 - other.0 > BigStride/2 {
            // overflow
            Some(Ordering::Less)
        } else if self.0 < other.0 && other.0 - self.0 > BigStride/2 {
            // overflow
            Some(Ordering::Greater)
        } else if self.0 < other.0 {
            Some(Ordering::Less)
        } else {
            Some(Ordering::Greater)
        }
    }
}
```
