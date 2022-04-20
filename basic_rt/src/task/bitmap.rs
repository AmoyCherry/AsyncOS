#![no_std]


use alloc::sync::Arc;

use spin::Mutex;
use lazy_static::*;

use bit_field::BitField;


#[no_mangle]
lazy_static! {
    pub static ref BITMAP: Arc<Mutex<BitMap>> = Arc::new(Mutex::new(BitMap::new()));
}

pub const PRIO_NUM: usize = 8;
const SYS_BITMAP_VA: usize = 0x8741_0000;
const USER_BITMAP_VA: usize = 0x8742_0000;

#[derive(Clone, Copy)]
pub struct  BitMap(usize);

impl BitMap {
    pub fn new() -> BitMap{
        let mut bitmap: &mut BitMap;
        unsafe{
            bitmap = &mut *(USER_BITMAP_VA as *mut BitMap);
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

    pub fn get_priority(&self) -> usize {
        for i in 0..PRIO_NUM {
            if self.0.get_bit(i){
                return i;
            }
        }

        PRIO_NUM
    }

    pub fn get_user_bitmap() -> BitMap { unsafe { *(USER_BITMAP_VA as *const BitMap) } }
    pub fn get_sys_bitmap() -> BitMap { unsafe { *(SYS_BITMAP_VA as *mut BitMap) } }

    pub fn inner(&mut self) -> &mut usize {
        &mut self.0
    }
}

pub fn update_user_bitmap(prio: usize, val: bool) {
    unsafe {
        let bitmap = &mut *(USER_BITMAP_VA as *mut BitMap);
        bitmap.set(prio, val);
        /* if val { println!("add task, user bitmap:  {:#b}", bitmap.0); }
        else { println!("pop task, user bitmap:  {:#b}", bitmap.0); } */
    }
}

pub fn check_bitmap_should_yield() -> bool{
    unsafe{
        let sys_bitmap = unsafe { *(SYS_BITMAP_VA as *const BitMap) };

        let user_bitmap = BitMap::get_user_bitmap();

        //println!("check, user bitmap: {:#b}, sys bitmap: {:#b}", user_bitmap.0, sys_bitmap.0);

        if sys_bitmap.get_priority() < user_bitmap.get_priority(){
            return true;
        }
    }
    false
}