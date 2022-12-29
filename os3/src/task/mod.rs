mod context;
mod switch;
#[allow(clippy::module_inception)]
mod task;

use crate::config::{MAX_APP_NUM, MAX_SYSCALL_NUM};
use crate::loader::{get_num_app, init_app_cx};
use crate::sync::UPSafeCell;
use crate::syscall::SYSCALL_MAP;
use crate::timer::get_time_us;
pub use context::TaskContext;
use lazy_static::*;
pub use switch::__switch;
pub use task::{TaskControlBlock, TaskInfo, TaskStatus};

pub struct TaskManager {
    num_app: usize,
    inner: UPSafeCell<TaskManagerInner>,
}

struct TaskManagerInner {
    tasks: [TaskControlBlock; MAX_APP_NUM],
    current_task: usize,
}

lazy_static! {
    pub static ref TASK_MANAGER: TaskManager = {
        let num_app = get_num_app();
        let mut tasks = [TaskControlBlock {
            task_cx: TaskContext::zero_init(),
            task_status: TaskStatus::UnInit,
            syscall_times: [0; 5],
            start_time: 0,
        }; MAX_APP_NUM];
        for (i, t) in tasks.iter_mut().enumerate().take(num_app) {
            t.task_cx = TaskContext::goto_restore(init_app_cx(i));
            t.task_status = TaskStatus::Ready;
        }
        TaskManager {
            num_app,
            inner: unsafe {
                UPSafeCell::new(TaskManagerInner {
                    tasks,
                    current_task: 0,
                })
            },
        }
    };
}

impl TaskManager {
    fn run_first_task(&self) -> ! {
        let mut inner = self.inner.exclusive_access();
        let ref mut task0 = inner.tasks[0];
        task0.task_status = TaskStatus::Running;
        task0.start_time = get_time_us();
        let next_task_cx_ptr = &task0.task_cx as *const TaskContext;
        drop(inner);
        let mut _unused = TaskContext::zero_init();
        unsafe { __switch(&mut _unused as *mut TaskContext, next_task_cx_ptr) }
        panic!("Unreachable in run_first_task!");
    }
    fn mark_current_suspended(&self) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].task_status = TaskStatus::Ready;
    }
    fn mark_current_exited(&self) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].task_status = TaskStatus::Exited;
    }
    fn find_next_task(&self) -> Option<usize> {
        let inner = self.inner.exclusive_access();
        let current = inner.current_task;
        (current + 1..current + self.num_app + 1)
            .map(|id| id % self.num_app)
            .find(|id| inner.tasks[*id].task_status == TaskStatus::Ready)
    }
    fn run_next_task(&self) {
        if let Some(next) = self.find_next_task() {
            let mut inner = self.inner.exclusive_access();
            let current = inner.current_task;
            let ref mut next_task = inner.tasks[next];
            next_task.task_status = TaskStatus::Running;
            if next_task.start_time == 0 {
                next_task.start_time = get_time_us()
            }
            inner.current_task = next;
            let current_task_cx_ptr = &mut inner.tasks[current].task_cx as *mut TaskContext;
            let next_task_cx_ptr = &inner.tasks[next].task_cx as *const TaskContext;
            drop(inner);
            unsafe {
                __switch(current_task_cx_ptr, next_task_cx_ptr);
            }
        } else {
            panic!("All applications completed!");
        }
    }
    fn inc_syscall_for_current_task(&self, syscall_id_idx: usize) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].syscall_times[syscall_id_idx] += 1;
    }
    fn update_task_info(&self, ti: &mut TaskInfo) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        let current_task = inner.tasks[current];
        for (idx, call_id) in SYSCALL_MAP.iter().enumerate() {
            ti.syscall_times[*call_id] = current_task.syscall_times[idx]
        }
        ti.time = (get_time_us() - current_task.start_time) / 1000;
        ti.status = current_task.task_status;
    }
}

pub fn run_first_task() {
    TASK_MANAGER.run_first_task();
}

fn run_next_task() {
    TASK_MANAGER.run_next_task();
}

fn mark_current_suspended() {
    TASK_MANAGER.mark_current_suspended();
}

fn mark_current_exited() {
    TASK_MANAGER.mark_current_exited();
}

pub fn suspend_current_and_run_next() {
    mark_current_suspended();
    run_next_task();
}

pub fn exit_current_and_run_next() {
    mark_current_exited();
    run_next_task();
}

pub fn inc_syscall_for_current_task(idx: usize) {
    if idx < 5 {
        TASK_MANAGER.inc_syscall_for_current_task(idx);
    }
}

pub fn update_task_info(ti: &mut TaskInfo) {
    TASK_MANAGER.update_task_info(ti);
}
