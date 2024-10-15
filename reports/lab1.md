# Chapter 3 实验报告

---

## 荣誉守则
1. 在完成本次实验的过程（含此前学习的过程）中，我曾分别与 以下各位 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：

《你交流的对象说明》

2. 此外，我也参考了 以下资料 ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：

《你参考的资料说明》

3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。

---

## 1.实现功能

### My Code

```rust
// os/src/syscall/process.rs

/// YOUR JOB: Finish sys_task_info to pass testcases
pub fn sys_task_info(_ti: *mut TaskInfo) -> isize {
    // my code
    use crate::task::TASK_MANAGER;
    unsafe {
        (*_ti).status = TaskStatus::Running;
        let t = TASK_MANAGER.get_task_info(TASK_MANAGER.get_task_id());
        (*_ti).syscall_times = t.0;
        (*_ti).time = get_time_ms() - t.1;
    }
    info!("kernel: sys_task_info");
    0
    // my code
  
    //trace!("kernel: sys_task_info");
    //-1
}
```



```rust
// os/src/trap/mod.rs

/// trap handler
#[no_mangle]
pub fn trap_handler(cx: &mut TrapContext) -> &mut TrapContext {
    let scause = scause::read(); // get trap cause
    let stval = stval::read(); // get extra value
                               // trace!("into {:?}", scause.cause());
    match scause.cause() {
        Trap::Exception(Exception::UserEnvCall) => {
            // jump to next instruction anyway
            cx.sepc += 4;

            // my code
            use crate::task::TASK_MANAGER;
            TASK_MANAGER.update_task_info(cx.x[17]);
            // my code

            // get system call return value
            cx.x[10] = syscall(cx.x[17], [cx.x[10], cx.x[11], cx.x[12]]) as usize;
        }
        //..
    }
    cx
}
```

```rust
// os/src/task/mod.rs

use crate::config::{MAX_APP_NUM, MAX_SYSCALL_NUM};

		fn run_first_task(&self) -> ! {
        let mut inner = self.inner.exclusive_access();
      
        // my code
        inner.task_first_run[0] = get_time_ms() as i64;
        // my code
      
        //..
    }

		/// Switch current `Running` task to the task we have found,
    /// or there is no `Ready` task and we can exit with all applications completed
    fn run_next_task(&self) {
        if let Some(next) = self.find_next_task() {
            let mut inner = self.inner.exclusive_access();
            let current = inner.current_task;
            inner.tasks[next].task_status = TaskStatus::Running;
            inner.current_task = next;
          
          	// my code
            if inner.task_first_run[next] == -1 {
                inner.task_first_run[next] = get_time_ms() as i64;
            }
            // my code
          
            //..
        } else {
            panic!("All applications completed!");
        }
    }

/// Inner of Task Manager
pub struct TaskManagerInner {
    /// task list
    tasks: [TaskControlBlock; MAX_APP_NUM],
    /// id of current `Running` task
    current_task: usize,

    // my code
    table_task_info: [[u32; MAX_SYSCALL_NUM]; MAX_APP_NUM],
    task_first_run:[i64; MAX_APP_NUM],
    // my code
}

lazy_static! {
    /// Global variable: TASK_MANAGER
    pub static ref TASK_MANAGER: TaskManager = {
        //..
        TaskManager {
            num_app,
            inner: unsafe {
                UPSafeCell::new(TaskManagerInner {
                    tasks,
                    current_task: 0,
                  	// my code
                    table_task_info: [[0;MAX_SYSCALL_NUM]; MAX_APP_NUM], // init table
                    task_first_run:[-1; MAX_APP_NUM],
                    // my code
                })
            },
        }
    };
}

impl TaskManager {
    //..

    // my code
    /// get_task_id
    pub fn get_task_id(&self) -> usize {
        //println!("get task id");
        self.inner.exclusive_access().current_task
    }

    /// update_task_info
    pub fn update_task_info(&self, syscall_id: usize) {
        //println!("update task info");
        let task_id = self.inner.exclusive_access().current_task;
        self.inner.exclusive_access().table_task_info[task_id][syscall_id] += 1;
    }

    /// get_task_info
    pub fn get_task_info(&self, task_id: usize) -> ([u32; 500], usize) {
        //println!("get task info");
        let task_info = self.inner.exclusive_access().table_task_info[task_id];
        let task_first_run = self.inner.exclusive_access().task_first_run[task_id] as usize;
        (task_info, task_first_run)
    }
    // my code
}
```



## 2.问答题

1. 正确进入 U 态后，程序的特征还应有：使用 S 态特权指令，访问 S 态寄存器后会报错。 请同学们可以自行测试这些内容（运行 [三个 bad 测例 (ch2b_bad_*.rs)](https://github.com/LearningOS/rCore-Tutorial-Test-2024A/tree/master/src/bin) ）， 描述程序出错行为，同时注意注明你使用的 sbi 及其版本。

Todo

2. 深入理解 [trap.S](https://github.com/LearningOS/rCore-Camp-Code-2024A/blob/ch3/os/src/trap/trap.S) 中两个函数 `__alltraps` 和 `__restore` 的作用，并回答如下问题:

2.1 L40：刚进入 `__restore` 时，`a0` 代表了什么值。请指出 `__restore` 的两种使用情景。

Todo

2.2 L43-L48：这几行汇编代码特殊处理了哪些寄存器？这些寄存器的的值对于进入用户态有何意义？请分别解释。

```assembly
ld t0, 32*8(sp)
ld t1, 33*8(sp)
ld t2, 2*8(sp)
csrw sstatus, t0
csrw sepc, t1
csrw sscratch, t2
```

Todo

3. L50-L56：为何跳过了 `x2` 和 `x4`？

```assembly
ld x1, 1*8(sp)
ld x3, 3*8(sp)
.set n, 5
.rept 27
    LOAD_GP %n
    .set n, n+1
.endr
```

Todo

4. L60：该指令之后，`sp` 和 `sscratch` 中的值分别有什么意义？

```assembly
csrrw sp, sscratch, sp
```

Todo

5. `__restore`：中发生状态切换在哪一条指令？为何该指令执行之后会进入用户态？

Todo

6. L13：该指令之后，`sp` 和 `sscratch` 中的值分别有什么意义？

```assembly
csrrw sp, sscratch, sp
```

Todo

7. 从 U 态进入 S 态是哪一条指令发生的？

Todo
