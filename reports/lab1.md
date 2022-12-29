# 实验报告

最开始，抄写os3-ref的代码，完成了基本框架。

然后，尝试将`TaskInfo`加入到`TaskControlBlock`中，然后添加`inc_syscall_for_current_task`和`get_task_info`方法。
在`TaskManager`的方法中，添加更新`TaskInfo`的状态和时间的代码，
`inc_syscall_for_current_task`用于更新`TaskInfo`中的`syscall_times`计数，`get_task_info`用于返回`TaskControlBlock`
中的`TaskInfo`；然而启动测试之后，直接卡在了`[kernel] Hello, world!`不再变化。

然后猜测是内存不足，u32x500=4*500=2k 个字节，如果64位对齐一下，就是4k个字节了。
于是将`TaskInfo`的字段打散分`TaskControlBlock`中，`syscall_times`变为100个长度，还是启动不了。
接着想到，这个实验只需要计数5个系统调用，不如就将`syscall_times`变为5个长度，再反向映射一下。
然后启动测试，能运行了，但是`ch3_taskinfo`直接27行报错，几经探寻，发现是单位问题，我在`TaskManager`的方法中直接使用了`get_time`函数，而`ch3_taskinfo`的`get_time`是转换成`ms`单位了的，因此将`TaskManager`的方法中的`get_time`换成`get_time_us`，`get_task_info`改为`update_task_info`，其中`ti.time`除以1000，即为`ms`单位。


再次运行测试，至此lab1通过。
