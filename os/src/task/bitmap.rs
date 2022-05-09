use bit_field::BitField;
use spin::Mutex;
use alloc::boxed::Box;
use alloc::sync::Arc;

use alloc::{vec::Vec, vec};

use super::PRIO_PIDS;

const PRIO_NUM: usize = 8;
const SYS_BITMAP_VA: usize = 0x8741_0000;

#[derive(Clone, Copy)]
pub struct  BitMap(pub usize);

impl BitMap {
    pub fn new() -> BitMap {
        let mut bitmap: &mut BitMap;
        unsafe{
            // bitmap = &mut *(0x87810000 as *mut BitMap);
            bitmap = &mut *(SYS_BITMAP_VA as *mut BitMap);
        }
        bitmap.0 = 0;
        *bitmap.deref()
    }
    
    pub fn set(&mut self, id: usize, value:bool) {
        self.0.set_bit(id, value);
    }

    pub fn get(&mut self, id: usize) -> bool {
        self.0.get_bit(id)
    }

    pub fn get_priority(&mut self, id: usize) -> usize {
        for i in 0..PRIO_NUM {
            if self.0.get_bit(i){
                return i;
            }
        }
        PRIO_NUM
    }
    
    pub fn get_sys_bitmap() -> BitMap { unsafe { *( SYS_BITMAP_VA as *mut BitMap) } }

    pub fn inner(&mut self) -> &mut usize {
        &mut self.0
    }
}

/// return the first 1 mask of num's bytes
pub fn get_right_one_mask(num: usize) -> usize {
    let mut pos: usize = 1;
    for i in 0..PRIO_NUM {
        if (num & pos) != 0 { return pos; }
        pos = (pos << 1);
    }
    0
}


use crate::config::PAGE_SIZE;
use lazy_static::*;

/* #[no_mangle]
lazy_static! {
    pub static ref KERNEL_BITMAP: Arc<Mutex<Box<BitMap>>> = Arc::new(Mutex::new(Box::new(BitMap::new())));
} */

#[no_mangle]
lazy_static! {
    pub static ref KERNEL_BITMAP: Arc<Mutex<BitMap>> = Arc::new(Mutex::new(BitMap::new()));
}

const PROCESS_NUM: usize = 8;

pub fn update_bitmap(){
    let start_addr = 0x8740_0000 as usize;
    let mut ans = 0;
    let mut u_maps:Vec<usize> = vec![0];
    for i in 1..=PROCESS_NUM {
        let user_bitmap = unsafe { &*( (start_addr + PAGE_SIZE*i) as *const BitMap) };

        //debug!("[hart {}] update, [space {}] bitmap:    {:#b}",hart_id(), i, user_bitmap.0);
        //println!("[hart {}] update, [space {}] bitmap:    {:#b}",hart_id(), i, user_bitmap.0);
        u_maps.push(user_bitmap.0);
        ans = (ans | user_bitmap.0);
    }

    let mask = get_right_one_mask(ans);
    PRIO_PIDS.lock().clear();
    for i in 1..=PROCESS_NUM {
        if (u_maps[i] & mask) != 0 { 
            //debug!("PRIO_PIDS push: {}", i);
            PRIO_PIDS.lock().insert(i); 
        }
    }

    unsafe {
        // 要依靠裸指针修改内存值, 必须使用这种写法
        let sys_bitmap = &mut *(SYS_BITMAP_VA as *mut BitMap);
        sys_bitmap.0 = ans;
        //debug!("[hart {}] hard [{}] update bitmap :     {:#b}", hart_id(), crate::hart_id(), sys_bitmap.0);
    }
}


pub fn hart_id() -> usize {
    let hart_id: usize;
    unsafe {
        asm!("mv {}, tp", out(reg) hart_id);
    }
    hart_id
}


