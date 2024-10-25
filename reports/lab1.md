# Lab 1 实验报告

## 荣誉守则

1. 在完成本次实验的过程（含此前学习的过程）中，我曾分别与 **以下各位** 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：

   > 

2. 此外，我也参考了 **以下资料** ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：

   > *rCore-Camp-Guide-2024A 文档*

3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。



## 一 实现功能

1. sys_task_info

(1) 数据结构

在 `TaskControlBlock` 中添加字段，用于记录当前任务的执行情况:

```rust
syscall_info: [u32; MAX_SYSCALL_NUM] // 用于记录该任务系统调用的信息
first_run: usize // 用于记录该任务第一次运行的时间
```

(2) 维护数据结构

添加如下方法:

```rust
fn update_task_info(&self, syscall_id: usize)
fn get_task_info(&self) -> ([u32; 500], usize)
```

在 `TaskManager` 的 `run_first_task` 方法 和 `run_next_task` 方法中，记录待运行的任务的 `first_run` 字段

在 `trap_handler` 方法中，根据参数 syscall_id (记录在 `TrapContext[17]` 中) ，运行 `TASK_MANAGER` 的 `update_task_info` 方法，更新当前执行的任务的系统调用的信息

(3) 系统调用

运行 `TASK_MANAGER` 的 `get_task_info` 方法获得当前任务的执行情况，根据参数填入相关信息即可



## 二 问答作业

1.正确进入 U 态后，程序的特征还应有：使用 S 态特权指令，访问 S 态寄存器后会报错。 请同学们可以自行测试这些内容（运行 [三个 bad 测例 (ch2b_bad_*.rs)](https://github.com/LearningOS/rCore-Tutorial-Test-2024A/tree/master/src/bin) ）， 描述程序出错行为，同时注意注明你使用的 sbi 及其版本。

(1) ch2b_bad_address.rs:

报错：[kernel] PageFault in application, bad addr = 0x0, bad instruction = 0x804003a4, kernel killed it.

程序试图访问不属于其地址空间的 0x0 地址，陷入到 S 态，被 OS 杀死

(2) ch2b_bad_instructions.rs

报错：[kernel] IllegalInstruction in application, kernel killed it.

程序试图使用特权指令 `sret`，陷入到 S 态，被 OS 杀死

(3) ch2b_bad_register.rs

报错：[kernel] IllegalInstruction in application, kernel killed it.

程序试图使用读取 `sstatus` 寄存器，陷入到 S 态，被 OS 杀死

sbi: RustSBI version 0.3.0-alpha.2



2.深入理解 [trap.S](https://github.com/LearningOS/rCore-Camp-Code-2024A/blob/ch3/os/src/trap/trap.S) 中两个函数 `__alltraps` 和 `__restore` 的作用，并回答如下问题:

2.1 L40：刚进入 `__restore` 时，`a0` 代表了什么值。请指出 `__restore` 的两种使用情景。

(1) a0 代表内核栈栈顶

(2) 运行用户程序的时候，需要使用 `__restore` 从内核态切换到用户态；用户程序发起系统调用之后，需要使用 `__restore` 从内核态切换到用户态。



2.2 L43-L48：这几行汇编代码特殊处理了哪些寄存器？这些寄存器的的值对于进入用户态有何意义？请分别解释。

```assembly
ld t0, 32*8(sp)
ld t1, 33*8(sp)
ld t2, 2*8(sp)
csrw sstatus, t0
csrw sepc, t1
csrw sscratch, t2
```

在 `__alltraps` 中，这三个寄存器的值被保存在内核栈，这六行代码用于恢复 `sstatus` `sepc` `sscratch `这三个寄存器的值

`sstatus `保存了 trap 前的特权级，用于恢复用户态；`sepc` 保存了 trap 之后的下一条指令的地址，用于返回用户态的执行流；`sscratch` 保存了 trap 前用户栈的栈顶指针，用于恢复 sp 寄存器；



2.3 L50-L56：为何跳过了 `x2` 和 `x4`？

```assembly
ld x1, 1*8(sp)
ld x3, 3*8(sp)
.set n, 5
.rept 27
   LOAD_GP %n
   .set n, n+1
.endr
```

`x2`寄存器当前保存的是内核栈的栈顶指针，不需要保存，`x4`寄存器一般不会被用到，不需要保存



2.4 L60：该指令之后，`sp` 和 `sscratch` 中的值分别有什么意义？

```assembly
csrrw sp, sscratch, sp
```

`sp`寄存器和`sscratch`寄存器的值会被交换

在 `__restore` 中，该指令会使得 `sp` 指向用户栈，`sscratch` 指向内核栈。接下来就可以回到用户态



2.5 `__restore`：中发生状态切换在哪一条指令？为何该指令执行之后会进入用户态？

发生在 `sret` 指令；`sret` 指令属于 `eret` (一类执行环境返回指令)，`sret` 指令特指从 Supervisor 模式的执行环境（即 OS 内核）返回的指令



2.6 L13：该指令之后，`sp` 和 `sscratch` 中的值分别有什么意义？

```assembly
csrrw sp, sscratch, sp
```

`sp`寄存器和`sscratch`寄存器的值会被交换

在 `__alltraps` 中，该指令会使得 `sp` 指向内核栈，`sscratch` 指向用户栈。接下来就可以使用 `sp` 在内核栈上保存 `TrapContext`



2.7 从 U 态进入 S 态是哪一条指令发生的？

`ecall` 指令执行后，CPU 会跳转到 `stvec` 所设置的 Trap 处理入口地址，并将当前特权级设置为 S ，然后从Trap 处理入口地址处开始执行。