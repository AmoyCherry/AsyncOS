use core::mem::transmute;
use crate::fs::{open_file, OpenFlags};
use xmas_elf::sections::SectionData::{self, SymbolTable64};
use xmas_elf::symbol_table::{Entry, Entry64};
use xmas_elf::{header, program::Type, ElfFile};
use xmas_elf::sections::{SectionHeader, SectionHeader_};
pub type P64 = u64;
use crate::mm::KERNEL_SPACE;



pub fn init(){
    println!("lkm init");
    add_lkm_image();
    println!("lkm init done");
}



pub fn add_lkm_image(){
    let inode = open_file("basic_rt", OpenFlags::RDONLY).unwrap();
    let mut v = inode.read_all();
    let elf = ElfFile::new(&v).unwrap();


    println!("len {:#x?}", v.len());
    // for section in elf.section_iter(){
    //     println!("section {:?}", section.get_name(&elf));
    // }
    let mut entry = 0 as usize;

    for sym  in symbol_table(&elf){
        let name = sym.get_name(&elf);
        let s = "demo_add";
        if name.unwrap() == s{
            println!("name {:?}  value:{:#x?}", name, sym.value());
            entry = sym.value() as usize;
        }
    }

    let base = 0x86000000 as usize;
    KERNEL_SPACE.lock().add_lkm(v.as_slice(), base);
    // KERNEL_SPACE.lock().init_module(v.as_slice());
    KERNEL_SPACE.lock().activate();

    println!("map tiny_kernel done");
    // unsafe {
    //     let demo_add: unsafe extern "C" fn(usize, usize)->usize = transmute(entry + base);
    //     println!("[LKM] calling demo_add at {:?}", demo_add);
    //     let x:usize = demo_add(2, 3);
    //     println!("[LKM] demo_add(2,3) returned {:?}", x);
    // }
}

fn symbol_table<'a>(elf: &ElfFile<'a>) -> &'a [Entry64] {
    match elf.find_section_by_name(".symtab").unwrap().get_data(&elf).unwrap()
    {
        SymbolTable64(dsym) => dsym,
        _ => panic!("corrupted .symtab"),
    }
}


pub fn get_symbol_addr_from_elf(file_name: &str, symbol_name: &str) -> usize{
    let inode = open_file(file_name, OpenFlags::RDONLY).unwrap();
    let mut v = inode.read_all();
    let elf = ElfFile::new(&v).unwrap();
    let mut entry = 0 as usize;
    for sym  in symbol_table(&elf){
        let name = sym.get_name(&elf);
        if name.unwrap() == symbol_name{
            // println!("name {:?}  value:{:#x?}", name, sym.value());
            entry = sym.value() as usize;
        }
    }
    entry
}