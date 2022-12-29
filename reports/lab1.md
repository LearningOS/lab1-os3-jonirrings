# 实验报告
最开始 ，抄写os3-ref的代码，完成了基本框架。


然后，尝试将`TaskInfo`加入到`TaskControlBlock`中，然后添加`inc_syscall_for_current_task`和`get_task_info`方法。
在`TaskManager`的方法中，添加更新`TaskInfo`的状态和时间的代码，
`inc_syscall_for_current_task`用于更新`TaskInfo`中的`syscall_times`计数，`get_task_info`用于返回`TaskControlBlock`
中的`TaskInfo`；然而启动测试之后，直接卡在了`[kernel] Hello, world!`不再变化。


然后猜测是内存不足，u32x500=4*500=2k 个字节，如果64位对齐一下，就是4k个字节了。
于是将`TaskInfo`的字段打散分`TaskControlBlock`中，`syscall_times`变为100个长度，还是启动不了。
接着想到，这个实验只需要计数5个系统调用，不如就将`syscall_times`变为5个长度，再反向映射一下。
然后启动测试，能运行了，但是`ch3_taskinfo`直接27行报错，几经探寻，发现是单位问题，我在`TaskManager`的方法中直接使用了`get_time`函数，而`ch3_taskinfo`的`get_time`是转换成`ms`单位了的，因此将`TaskManager`的方法中的`get_time`换成`get_time_us`，`get_task_info`改为`update_task_info`，其中`ti.time`除以1000，即为`ms`单位。


再次运行测试，至此lab1通过。

断断续续历时3天，总算完成了第一个实操的实验。


# 问答
1. 红色报错信息
   ```text
   [ERROR] [kernel] PageFault in application, bad addr = 0x0, bad instruction = 0x80400410, core dumped.
   [ERROR] [kernel] IllegalInstruction in application, core dumped.
   [ERROR] [kernel] IllegalInstruction in application, core dumped.
   ```
2. 两个函数的作用是进入和退出`S`模式时候的环境准备与还原，中间调用`trap_handler`处理中断/异常。
   1. 刚进入`__restore`亦即从`trap_handler`返回，`a0`未修改，应还是指向了`TrapContext`。
      `__restore`的主要使用场景就是退出`S`模式，还原堆栈指向；也就是实现特权级切换和还原调用现场。
   2. `csrw sstatus, t0`，还原`sstatus`寄存器，还原中断使能。
      `csrw sepc, t1`，还原`sepc`寄存器，亦即`sret`退出时跳转的中断/异常发生地点。
      `csrw sscratch, t2`，还原`sscratch`寄存器，用于还原中断/异常发生前的`a0`值。
   3. `x2`是`sp`，后续保存到了`a0`中，为构造的`TrapContext`，`x4`是`tp`亦即`Thread Pointer`，
      代码中没有使用到线程，暂时不处理它。
   4. L63之前`sp`上先释放了`TrapContext`的内存，然后此时`sp`指向的内核空间，`sscratch`指向用户空间，
      `csrrw sp, sscratch, sp`之后，`sp`指向用户空间，`sscratch`指向内核空间。
   5. `sret`，根据手册，此指令的执行，`pc`将指向`sepc`，一起一些权限处理；
      且此句之前，`sp`已换回用户空间指向，因此就转变到到用户权限和用户空间。
   6. 与`__restore`末为的呼应，将`sscratch`和`sp`互换，
      这之前`sscratch`指向的是内核空间，交换之后`sp`指向内核空间，开始做内和空间上的异常/中断处理。
   7. `sbi.rs`的`sbi_call`函数中的`ecall`汇编指令
