const SYSCALL_WRITE: usize = 64;
const SYSCALL_EXIT: usize = 93;
const SYSCALL_YIELD: usize = 124;
const SYSCALL_GET_TIME: usize = 169;
const SYSCALL_TASK_INFO: usize = 410;

pub const SYSCALL_MAP: [usize; 5] = [
    SYSCALL_WRITE,
    SYSCALL_EXIT,
    SYSCALL_YIELD,
    SYSCALL_GET_TIME,
    SYSCALL_TASK_INFO,
];

mod fs;
mod process;

use crate::task::{inc_syscall_for_current_task, TaskInfo};
use fs::*;
use process::*;

pub fn syscall(syscall_id: usize, args: [usize; 3]) -> isize {
    if let Some(idx) = SYSCALL_MAP.iter().position(|x| *x == syscall_id) {
        inc_syscall_for_current_task(idx);
    }
    match syscall_id {
        SYSCALL_WRITE => sys_write(args[0], args[1] as *const u8, args[2]),
        SYSCALL_EXIT => sys_exit(args[0] as i32),
        SYSCALL_YIELD => sys_yield(),
        SYSCALL_GET_TIME => sys_get_time(args[0] as *mut TimeVal, args[1]),
        SYSCALL_TASK_INFO => sys_task_info(args[0] as *mut TaskInfo),
        _ => panic!("Unsupported syscall_id: {}", syscall_id),
    }
}
