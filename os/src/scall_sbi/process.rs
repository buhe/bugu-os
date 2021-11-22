// use crate::loader::get_app_data_by_name;
// use crate::mmu::{translated_refmut, translated_str};
// // use crate::task::{
// //     add_task, current_task, current_user_token, exit_current_and_run_next,
// //     suspend_current_and_run_next,
// // };
// use crate::timer::get_time_ms;
// use alloc::sync::Arc;

pub fn sys_exit(exit_code: i32) -> ! {
    // println!("[kernel] Application exited with code {}", exit_code);
    // exit_current_and_run_next(exit_code);
    panic!("Unreachable in sys_exit!");
}

pub fn sys_yield() -> isize {
    // suspend_current_and_run_next();
    0
}

pub fn sys_get_time() -> isize {
    // get_time_ms() as isize
    0
}

pub fn sys_getpid() -> isize {
    // current_task().unwrap().pid.0 as isize
    0
}

pub fn sys_fork() -> isize {
    // 
    0
}

pub fn sys_exec(path: *const u8) -> isize {
    // let token = current_user_token();
    // let path = translated_str(token, path);
    // if let Some(data) = get_app_data_by_name(path.as_str()) {
    //     let task = current_task().unwrap();
    //     task.exec(data);
    //     0
    // } else {
        -1
    // }
}

/// If there is not a child process whose pid is same as given, return -1.
/// Else if there is a child process but it is still running, return -2.
pub fn sys_waitpid(pid: isize, exit_code_ptr: *mut i32) -> isize {
    // let task = current_task().unwrap();
    // // find a child process

    // // ---- hold current PCB lock
    // let mut inner = task.acquire_inner_lock();
    // if inner
    //     .children
    //     .iter()
    //     .find(|p| pid == -1 || pid as usize == p.getpid())
    //     .is_none()
    // {
    //     return -1;
    //     // ---- release current PCB lock
    // }
    // let pair = inner.children.iter().enumerate().find(|(_, p)| {
    //     // ++++ temporarily hold child PCB lock
    //     p.acquire_inner_lock().is_zombie() && (pid == -1 || pid as usize == p.getpid())
    //     // ++++ release child PCB lock
    // });
    // if let Some((idx, _)) = pair {
    //     let child = inner.children.remove(idx);
    //     // confirm that child will be deallocated after removing from children list
    //     assert_eq!(Arc::strong_count(&child), 1);
    //     let found_pid = child.getpid();
    //     // ++++ temporarily hold child lock
    //     let exit_code = child.acquire_inner_lock().exit_code;
    //     // ++++ release child PCB lock
    //     *translated_refmut(inner.memory_set.token(), exit_code_ptr) = exit_code;
    //     found_pid as isize
    // } else {
        -2
    // }
    // ---- release current PCB lock automatically
}
