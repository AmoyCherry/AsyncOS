use crate::task::{
    suspend_current_and_run_next,
    exit_current_and_run_next,
    current_task,
    current_user_token,
    add_task,
    // switch_to_spaceid,
};
use crate::timer::get_time_ms;
use crate::mm::{
    translated_str,
    translated_refmut,
    translated_ref,
};
use crate::fs::{
    open_file,
    OpenFlags,
};
use alloc::sync::Arc;
use alloc::vec::Vec;
use alloc::string::{String, ToString};

pub fn sys_exit(exit_code: i32) -> ! {
    crate::info!("user exit");
    exit_current_and_run_next(exit_code);
    panic!("Unreachable in sys_exit!");
}

pub fn sys_yield() -> isize {
    println!("[kernel] yield, suspend current and run next");
    suspend_current_and_run_next();
    0
}

pub fn sys_do_yield(space_id:usize) -> isize {
    // switch_to_spaceid(space_id);
    0
}


pub fn sys_get_time() -> isize {
    get_time_ms() as isize
}

pub fn sys_getpid() -> isize {
    current_task().unwrap().pid.0 as isize
}

pub fn sys_fork() -> isize {
    let current_task = current_task().unwrap();
    let new_task = current_task.fork();
    let new_pid = new_task.pid.0;
    // modify trap context of new_task, because it returns immediately after switching
    let trap_cx = new_task.acquire_inner_lock().get_trap_cx();
    // we do not have to move to next instruction since we have done it before
    // for child process, fork returns 0
    trap_cx.x[10] = 0;
    // add new task to scheduler
    add_task(new_task);
    new_pid as isize
}

pub fn sys_exec(path: *const u8, mut args: *const usize) -> isize {
    let token = current_user_token();
    let path = translated_str(token, path);

    let mut args_vec: Vec<String> = Vec::new();
    loop {
        let arg_str_ptr = *translated_ref(token, args);
        if arg_str_ptr == 0 {
            break;
        }
        args_vec.push(translated_str(token, arg_str_ptr as *const u8));
        unsafe { args = args.add(1); }
    }
    if let Some(app_inode) = open_file(path.as_str(), OpenFlags::RDONLY) {
        let all_data = app_inode.read_all();
        let task = current_task().unwrap();
        let argc = args_vec.len();
        task.exec(all_data.as_slice(), args_vec);
        // return argc because cx.x[10] will be covered with it later
        argc as isize
    } else {
        -1
    }
}


pub fn sys_exec_3() -> isize {
    let token = current_user_token();
    // let path = translated_str(token, path);
    let mut args_vec: Vec<String> = Vec::new();
    let mut args = 0 as *const u8;
    let path = String::from("3");
    loop {
        let arg_str_ptr = *translated_ref(token, args);
        if arg_str_ptr == 0 {
            break;
        }
        args_vec.push(translated_str(token, arg_str_ptr as *const u8));
        unsafe { args = args.add(1); }
    }
    if let Some(app_inode) = open_file(path.as_str(), OpenFlags::RDONLY) {
        let all_data = app_inode.read_all();
        let task = current_task().unwrap();
        let argc = args_vec.len();
        task.exec(all_data.as_slice(), args_vec);
        // return argc because cx.x[10] will be covered with it later
        argc as isize
    } else {
        -1
    }
}




/// If there is not a child process whose pid is same as given, return -1.
/// Else if there is a child process but it is still running, return -2.
pub fn sys_waitpid(pid: isize, exit_code_ptr: *mut i32) -> isize {
    let task = current_task().unwrap();
    // find a child process

    // ---- hold current PCB lock
    let mut inner = task.acquire_inner_lock();
    if inner.children
        .iter()
        .find(|p| {pid == -1 || pid as usize == p.getpid()})
        .is_none() {
        return -1;
        // ---- release current PCB lock
    }
    let pair = inner.children
        .iter()
        .enumerate()
        .find(|(_, p)| {
            // ++++ temporarily hold child PCB lock
            p.acquire_inner_lock().is_zombie() && (pid == -1 || pid as usize == p.getpid())
            // ++++ release child PCB lock
        });
    if let Some((idx, _)) = pair {
        let child = inner.children.remove(idx);
        // confirm that child will be deallocated after being removed from children list
        assert_eq!(Arc::strong_count(&child), 1);
        let found_pid = child.getpid();
        // ++++ temporarily hold child lock
        let exit_code = child.acquire_inner_lock().exit_code;
        // ++++ release child PCB lock
        *translated_refmut(inner.memory_set.token(), exit_code_ptr) = exit_code;
        found_pid as isize
    } else {
        -2
    }
    // ---- release current PCB lock automatically
}



pub fn sys_get_symbol_addr(symbol_name: *const u8) -> isize{
    // println!("sys_get_symbol_addr {:?}", symbol_name);
    let token = current_user_token();
    let name = translated_str(token, symbol_name);
    let name = name.as_str();
    let addr = crate::lkm::get_symbol_addr_from_elf("basic_rt", name);

    addr as isize

}



pub fn sys_shut_done() -> isize {
    panic!("shut done!");
    0
}

pub fn sys_get_satp() -> isize {
    current_user_token() as isize
}

// use crate::loader::get_app_data_by_name;

// pub fn sys_exec_v1() -> isize {
//     let token = current_user_token();
//     let path = "v1".to_string();
//     if let Some(data) = get_app_data_by_name(path.as_str()) {
//         let task = current_task().unwrap();
//         task.exec_v1(data);
//         0
//     } else {
//         -1
//     }
// }