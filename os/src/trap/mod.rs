pub mod context;

use riscv::{addr::BitField, register::{
    mtvec::TrapMode,
    stvec,
    scause::{
        self,
        Trap,
        Exception,
        Interrupt,
    },
    stval,
    sie,
    sstatus,
}};
use crate::{syscall::syscall, basic_rt::thread::cpu_run};
use crate::task::{
    exit_current_and_run_next,
    suspend_current_and_run_next,
    current_user_token,
    current_trap_cx,
    current_task,
};
use crate::timer::set_next_trigger;
use crate::config::{TRAP_CONTEXT, TRAMPOLINE};

global_asm!(include_str!("trap.S"));

pub fn init() {
    
    set_kernel_trap_entry();
}

fn set_kernel_trap_entry() {
    unsafe {
        stvec::write(trap_from_kernel as usize, TrapMode::Direct);
    }
}

fn set_user_trap_entry() {
    unsafe {
        stvec::write(TRAMPOLINE as usize, TrapMode::Direct);
    }
}

pub fn enable_timer_interrupt() {
    unsafe { sie::set_stimer(); }
}

#[no_mangle]
pub fn trap_handler() -> ! {
    set_kernel_trap_entry();
    let scause = scause::read();
    let stval = stval::read();
    match scause.cause() {
        Trap::Exception(Exception::UserEnvCall) => {
            // jump to next instruction anyway
            let mut cx = current_trap_cx();
            cx.sepc += 4;
            // get system call return value
            let result = syscall(cx.x[17], [cx.x[10], cx.x[11], cx.x[12], cx.x[13], cx.x[14], cx.x[15]]);
            // cx is changed during sys_exec, so we have to call it again
            cx = current_trap_cx();
            cx.x[10] = result as usize;



        }
        Trap::Exception(Exception::StoreFault) |
        Trap::Exception(Exception::StorePageFault) |
        Trap::Exception(Exception::InstructionFault) |
        Trap::Exception(Exception::InstructionPageFault) |
        Trap::Exception(Exception::LoadFault) |
        Trap::Exception(Exception::LoadPageFault) => {
            println!(
                "[kernel] {:?} in application, bad addr = {:#x}, bad instruction = {:#x}, core dumped.",
                scause.cause(),
                stval,
                current_trap_cx().sepc,
            );
            // page fault exit code
            exit_current_and_run_next(-2);
        }
        Trap::Exception(Exception::IllegalInstruction) => {
            println!("[kernel] IllegalInstruction in application, core dumped.");
            println!(
                "[kernel] {:?} in application, bad addr = {:#x}, bad instruction = {:#x}, core dumped.",
                scause.cause(),
                stval,
                current_trap_cx().sepc,
            );
            // illegal instruction exit code
            exit_current_and_run_next(-3);
        }
        Trap::Interrupt(Interrupt::SupervisorTimer) => {
            set_next_trigger();

            //crate::task::update_bitmap();
            cpu_run();
            
            // info!("[kernel] timer interrupt");
            suspend_current_and_run_next();
        }
        _ => {
            panic!("Unsupported trap {:?}, stval = {:#x}!", scause.cause(), stval);
        }
    }
    //println!("before trap_return");
    trap_return();
}


#[no_mangle]
pub fn trap_return() -> ! {
    unsafe {
        sstatus::clear_sie();
    }

    set_user_trap_entry();
    let trap_cx_ptr = TRAP_CONTEXT;
    let user_satp = current_user_token();

    extern "C" {
        fn __alltraps();
        fn __restore();
    }
    let restore_va = __restore as usize - __alltraps as usize + TRAMPOLINE;
    unsafe {
        asm!(
            "fence.i",
            "jr {restore_va}",
            restore_va = in(reg) restore_va,
            in("a0") trap_cx_ptr,
            in("a1") user_satp,
            options(noreturn)
        );
    }
}


#[no_mangle]
pub fn trap_return1(space_id:usize) -> ! {
    set_user_trap_entry();

    use riscv::register::{
        sstatus::{self, SPP},
        // stvec::{self, TrapMode},
    };
        // 设置 sstatus.SPP 的值为 User
    unsafe {
        sstatus::set_spp(SPP::User);
    }
    let trap_cx_ptr = crate::config::swap_contex_va(1);
    let user_satp = current_user_token();
    extern "C" {
        fn __alltraps();
        fn __restore();
    }
    let restore_va = __restore as usize - __alltraps as usize + TRAMPOLINE;
    unsafe {
        llvm_asm!("fence.i" :::: "volatile");
        llvm_asm!("jr $0" :: "r"(restore_va), "{a0}"(trap_cx_ptr), "{a1}"(user_satp) :: "volatile");
    }
    panic!("Unreachable in back_to_user!");
}



#[no_mangle]
pub fn trap_from_kernel() -> !{
    let stval = stval::read();
    let sepc = sepc_read();
    panic!("a trap {:?}  stval = {:#x}! sepc = {:#x} from kernel!", scause::read().cause(), stval, sepc);
}


pub fn sepc_read() -> usize {
    let ret: usize;
    unsafe {llvm_asm!("csrr $0, sepc":"=r"(ret):::"volatile");}
    ret
}
pub use context::{TrapContext};



pub unsafe fn get_swap_cx<'cx>(satp: usize, asid: usize) -> &'cx mut TrapContext {
    let root_ppn = satp.get_bits(0..44);
    let cx_va = crate::config::swap_contex_va(asid);
    let cx_pa = crate::mm::translated_context(satp, cx_va);
    (cx_pa as *mut TrapContext).as_mut().unwrap()
}


#[no_mangle]
pub fn switch_to_user(satp: usize, asid: usize) -> ! {
    set_user_trap_entry();
    let next_swap_contex = unsafe { get_swap_cx(satp, asid) };
    let trap_cx_ptr = crate::config::swap_contex_va(asid);
    let user_satp = satp;
    extern "C" {
        fn __alltraps();
        fn __restore();
    }
    let restore_va = __restore as usize - __alltraps as usize + TRAMPOLINE;
    unsafe {
        llvm_asm!("fence.i" :::: "volatile");
        llvm_asm!("jr $0" :: "r"(restore_va), "{a0}"(trap_cx_ptr), "{a1}"(user_satp) :: "volatile");
    }
    panic!("Unreachable in back_to_user!");
}

