use super::{PageTable, PageTableEntry, PTEFlags};
use super::{VirtPageNum, VirtAddr, PhysPageNum, PhysAddr};
use super::{FrameTracker, frame_alloc};
use super::{VPNRange, StepByOne};
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use riscv::register::satp;
use alloc::sync::Arc;
use lazy_static::*;
use spin::Mutex;
use crate::config::{
    MEMORY_END,
    PAGE_SIZE,
    TRAMPOLINE,
    TRAP_CONTEXT,
    USER_STACK_SIZE,
    MMIO,
};
use xmas_elf::program::Type::Load;
use crate::config::swap_contex_va;

use alloc::string::{String, ToString};
// use crate::loader::get_app_data_by_name;

extern "C" {
    fn stext();
    fn etext();
    fn srodata();
    fn erodata();
    fn sdata();
    fn edata();
    fn sbss_with_stack();
    fn ebss();
    fn ekernel();
    fn strampoline();
}

lazy_static! {
    pub static ref KERNEL_SPACE: Arc<Mutex<MemorySet>> = Arc::new(Mutex::new(
        MemorySet::new_kernel()
    ));
}

/* lazy_static! {
    pub static ref PID_2_BITMAP: Arc<Mutex<MemorySet>> = Arc::new(Mutex::new(
        MemorySet::new_kernel()
    ));
} */

const SYS_BITMAP_VA: usize = 0x8741_0000;
const USER_BITMAP_VA: usize = 0x8742_0000;


// lazy_static! {
//     pub static ref SPACE_ID_SATP : Vec<usize> = {
//         let mut v = Vec::new();
//         for i in 0..10{
//             v.push(0);
//         }
//         v
//     };
// }


pub fn kernel_token() -> usize {
    KERNEL_SPACE.lock().token()
}

pub struct MemorySet {
    page_table: PageTable,
    areas: Vec<MapArea>,
}

impl MemorySet {
    pub fn new_bare() -> Self {
        Self {
            page_table: PageTable::new(),
            areas: Vec::new(),
        }
    }
    pub fn token(&self) -> usize {
        self.page_table.token()
    }
    /// Assume that no conflicts.
    pub fn insert_framed_area(&mut self, start_va: VirtAddr, end_va: VirtAddr, permission: MapPermission) {
        self.push(MapArea::new(
            start_va,
            end_va,
            MapType::Framed,
            permission,
        ), None);
    }
    pub fn remove_area_with_start_vpn(&mut self, start_vpn: VirtPageNum) {
        if let Some((idx, area)) = self.areas.iter_mut().enumerate()
            .find(|(_, area)| area.vpn_range.get_start() == start_vpn) {
            area.unmap(&mut self.page_table);
            self.areas.remove(idx);
        }
    }
    fn push(&mut self, mut map_area: MapArea, data: Option<&[u8]>) {
        map_area.map(&mut self.page_table);
        if let Some(data) = data {
            map_area.copy_data(&mut self.page_table, data);
        }
        self.areas.push(map_area);
    }
    /// Mention that trampoline is not collected by areas.
    fn map_trampoline(&mut self) {

        self.page_table.map(
            VirtAddr::from(TRAMPOLINE).into(),
            PhysAddr::from(strampoline as usize).into(),
            PTEFlags::R | PTEFlags::X,
        );
        // info!("map_trampoline start:{:#x} end:{:#x}", TRAMPOLINE, strampoline as usize);
    }

    fn copy_from_kernel(&mut self, mut map_area: MapArea, data: &[u8]) {
        map_area.map(&mut self.page_table);
        map_area.copy_kernel_data(&mut self.page_table, data);

        self.areas.push(map_area);
    }


    // fn map_context(&mut self, space_id:usize) {
    //     let context_va = swap_contex_va(space_id);
    //     println!("map context_va {:#x}", context_va);
    //     self.page_table.map(
    //         VirtAddr::from(context_va).into(),  
    //         PhysAddr::from(context_va).into(),  
    //         PTEFlags::R | PTEFlags::X | PTEFlags::W |PTEFlags::U
    //     );
    // }

    //为内核模块设置空间,以及映射内核以及用户位图.

    pub fn push_shared_kernel(&mut self) {
        let start_addr = 0x87000000 as usize;
        for i in 0..(1024) {
            self.page_table.map(
                VirtAddr::from(start_addr + PAGE_SIZE*i).into(),  
                PhysAddr::from(start_addr + PAGE_SIZE*i).into(),  
                PTEFlags::R | PTEFlags::X | PTEFlags::W 
            );
        }
        for i in 0..(1024) {
            self.page_table.map(
                VirtAddr::from(0 + PAGE_SIZE*i).into(),  
                PhysAddr::from(start_addr + PAGE_SIZE*i).into(),  
                PTEFlags::R | PTEFlags::X | PTEFlags::W 
            );
        }
        println!("start: {:#x} end: {:#x}", start_addr, start_addr + PAGE_SIZE*(1024));

        
        let start_addr = 0x8740_0000 as usize;

        
        //user bitmap  kernel space  va = pa = 0x87400000 + page_size * pid 
        for i in 1..9{
            info!("kernel pid:{:#x} user bitmap in pa:{:#x}", i, start_addr + PAGE_SIZE*i);
            self.page_table.map(
                VirtAddr::from(start_addr + PAGE_SIZE * i).into(),  
                PhysAddr::from(start_addr + PAGE_SIZE * i).into(),  
                PTEFlags::R | PTEFlags::X|PTEFlags::W
            );
        }
        
        //kernel_bitmap  kernel space va = pa = 0x87410000 + page_size * pid 
        self.page_table.map(
            VirtAddr::from(SYS_BITMAP_VA).into(),  
            PhysAddr::from(SYS_BITMAP_VA).into(),  
            PTEFlags::R | PTEFlags::X |PTEFlags::W
        );

    }

    //为用户映射位图
    pub fn bitmap_user(&mut self, space_id: usize) {
        
        let start_addr = 0x8740_0000 as usize;
        info!("pid:{:#x} bitmap pa:{:#x}", space_id, start_addr + PAGE_SIZE*space_id);

        //user bitmap user space  va = 0x87420000  pa = 0x87400000 + page_size * pid
        self.page_table.map(
            VirtAddr::from(USER_BITMAP_VA).into(),  
            PhysAddr::from(start_addr + PAGE_SIZE*space_id).into(),  
            PTEFlags::R | PTEFlags::X  | PTEFlags::U |PTEFlags::W
        );

        //kernel bitmap  user space va = 0x87410000 pa = 0x87410000
        self.page_table.map(
            VirtAddr::from(SYS_BITMAP_VA).into(),  
            PhysAddr::from(SYS_BITMAP_VA).into(),  
            PTEFlags::R | PTEFlags::X  | PTEFlags::U |PTEFlags::W
        );
    }


    /// Without kernel stacks.
    pub fn new_kernel() -> Self {
        let mut memory_set = Self::new_bare();
        // map trampoline
        memory_set.map_trampoline();
        // map kernel sections
        println!(".text [{:#x}, {:#x})", stext as usize, etext as usize);
        println!(".rodata [{:#x}, {:#x})", srodata as usize, erodata as usize);
        println!(".data [{:#x}, {:#x})", sdata as usize, edata as usize);
        println!(".bss [{:#x}, {:#x})", sbss_with_stack as usize, ebss as usize);

        println!("ekernel  MEMORY_END [{:#x}, {:#x})", ekernel as usize, MEMORY_END as usize);

        println!("mapping .text section");
        memory_set.push(MapArea::new(
            (stext as usize).into(),
            (etext as usize).into(),
            MapType::Identical,
            MapPermission::R | MapPermission::X,
        ), None);
        println!("mapping .rodata section");
        memory_set.push(MapArea::new(
            (srodata as usize).into(),
            (erodata as usize).into(),
            MapType::Identical,
            MapPermission::R,
        ), None);
        println!("mapping .data section");
        memory_set.push(MapArea::new(
            (sdata as usize).into(),
            (edata as usize).into(),
            MapType::Identical,
            MapPermission::R | MapPermission::W,
        ), None);
        println!("mapping .bss section");
        memory_set.push(MapArea::new(
            (sbss_with_stack as usize).into(),
            (ebss as usize).into(),
            MapType::Identical,
            MapPermission::R | MapPermission::W,
        ), None);
        println!("mapping physical memory");
        memory_set.push(MapArea::new(
            (ekernel as usize).into(),
            (0x85ff0000).into(),
            MapType::Identical,
            MapPermission::R | MapPermission::W | MapPermission::X,
        ), None);

        memory_set.push_shared_kernel();
        // memory_set.bitmap_kernel();

        println!("mapping memory-mapped registers");
        for pair in MMIO {
            memory_set.push(MapArea::new(
                (*pair).0.into(),
                ((*pair).0 + (*pair).1).into(),
                MapType::Identical,
                MapPermission::R | MapPermission::W | MapPermission::X,
            ), None);
        }
        memory_set
    }
    
    
    /// Include sections in elf and trampoline and TrapContext and user stack,
    /// also returns user_sp and entry point.
    pub fn from_elf(elf_data: &[u8], space_id: usize) -> (Self, usize, usize) {

        let mut memory_set = Self::new_bare();

        let user_satp = memory_set.token();
        memory_set.map_trampoline();


        //shared
        let base = 0x86000000;
        use crate::fs::{open_file, OpenFlags};
        use xmas_elf::{ElfFile};
        let inode = open_file("basic_rt", OpenFlags::RDONLY).unwrap();
        let mut v = inode.read_all();
        let elfx = ElfFile::new(&v).unwrap();
        let elf_header = elfx.header;
        let magic = elf_header.pt1.magic;
        let ph_count = elf_header.pt2.ph_count();
        let mut max_end_vpn = VirtPageNum(0);
        let mut last_end_va = 0 as usize; 
        for i in 0..ph_count {
            let ph = elfx.program_header(i).unwrap();
            // println!("start_va {:#x?} end_va {:#x?} ph.virtual_addr {:#x?} ph.mem_size {:#x?}", ph.virtual_addr() as usize + base, (ph.virtual_addr() + ph.mem_size()) as usize + base, ph.virtual_addr(), ph.mem_size() as usize);
            if ph.get_type().unwrap() == xmas_elf::program::Type::Load {
                if (ph.virtual_addr() + ph.mem_size()) as usize + base <= last_end_va {
                    continue;
                }
                let mut start = (ph.virtual_addr() ) as usize + base - 0x87000000;
                let mut end = (ph.virtual_addr() + ph.mem_size()) as usize + base - 0x87000000;
                let mut size = 0 as usize;

                if start < last_end_va &&  end > last_end_va {
                    let diff = last_end_va - start;
                    // println!("diff :{:#x?}", diff);
                    assert!(diff < PAGE_SIZE);
                    start = last_end_va;
                    end = (ph.virtual_addr() + ph.mem_size()) as usize + base - diff;
                }                
                let start_va: VirtAddr = (start).into();
                let end_va: VirtAddr = (end).into();
                let mut map_perm = MapPermission::W | MapPermission::X | MapPermission::R | MapPermission::U;
                let ph_flags = ph.flags();
                let map_area = MapArea::new(
                    start_va,
                    end_va,
                    MapType::Framed,
                    map_perm,
                );
                max_end_vpn = map_area.vpn_range.get_end();
                // println!("ph.offset :{:#x?}   ph.file_size:{:#x?}", ph.offset(), ph.file_size());
                memory_set.push(
                    map_area,
                    Some(&elfx.input[ph.offset() as usize..(ph.offset() + ph.file_size()) as usize])
                );

                if ((ph.virtual_addr() + ph.mem_size()) as usize + base) % 0x1000 == 0 {
                    last_end_va = (ph.virtual_addr() + ph.mem_size()) as usize + base;
                }else{
                    last_end_va = (((ph.virtual_addr() + ph.mem_size()) as usize + base)/0x1000) * 0x1000 + 0x1000;
                }
            }
        }
        // println!("entry_point :{:#x?}", elfx.header.pt2.entry_point() as usize);
        //shared
        
        // map program headers of elf, with U flag
        let elf = xmas_elf::ElfFile::new(elf_data).unwrap();
        let elf_header = elf.header;
        let magic = elf_header.pt1.magic;
        assert_eq!(magic, [0x7f, 0x45, 0x4c, 0x46], "invalid elf!");
        let ph_count = elf_header.pt2.ph_count();
        let mut max_end_vpn = VirtPageNum(0);
        for i in 0..ph_count {
            let ph = elf.program_header(i).unwrap();
            if ph.get_type().unwrap() == xmas_elf::program::Type::Load {
                let start_va: VirtAddr = (ph.virtual_addr() as usize).into();
                let end_va: VirtAddr = ((ph.virtual_addr() + ph.mem_size()) as usize).into();
                let mut map_perm = MapPermission::U;
                let ph_flags = ph.flags();
                if ph_flags.is_read() { map_perm |= MapPermission::R; }
                if ph_flags.is_write() { map_perm |= MapPermission::W; }
                if ph_flags.is_execute() { map_perm |= MapPermission::X; }
                let map_area = MapArea::new(
                    start_va,
                    end_va,
                    MapType::Framed,
                    map_perm,
                );
                max_end_vpn = map_area.vpn_range.get_end();
                memory_set.push(
                    map_area,
                    Some(&elf.input[ph.offset() as usize..(ph.offset() + ph.file_size()) as usize])
                );
            }
        }
        // map user stack with U flags
        let max_end_va: VirtAddr = max_end_vpn.into();
        let mut user_stack_bottom: usize = max_end_va.into();
        // guard page
        user_stack_bottom += PAGE_SIZE;
        let user_stack_top = user_stack_bottom + USER_STACK_SIZE;
        memory_set.push(MapArea::new(
            user_stack_bottom.into(),
            user_stack_top.into(),
            MapType::Framed,
            MapPermission::R | MapPermission::W | MapPermission::U,
        ), None);
        // map TrapContext
        memory_set.push(MapArea::new(
            TRAP_CONTEXT.into(),
            TRAMPOLINE.into(),
            MapType::Framed,
            MapPermission::R | MapPermission::W,
        ), None);

        memory_set.push_shared();

        memory_set.bitmap_user(space_id);
        

        
        (memory_set, user_stack_top, elf.header.pt2.entry_point() as usize)
    }

    pub fn push_shared(&mut self) {
        let start_addr = 0x87000000 as usize;
        for i in 0..1024 {
            self.page_table.map(
                VirtAddr::from(start_addr + PAGE_SIZE*i).into(),  
                PhysAddr::from(start_addr + PAGE_SIZE*i).into(),  
                PTEFlags::R | PTEFlags::X  | PTEFlags::U |PTEFlags::W
            );
        }
    }





    pub fn from_elf_no_use(elf_data: &[u8], space_id: usize) -> (Self, usize, usize) {

        let mut memory_set = Self::new_bare();

        let user_satp = memory_set.token();

        // map program headers of elf, with U flag
        let elf = xmas_elf::ElfFile::new(elf_data).unwrap();
        let elf_header = elf.header;
        let magic = elf_header.pt1.magic;
        assert_eq!(magic, [0x7f, 0x45, 0x4c, 0x46], "invalid elf!");
        let ph_count = elf_header.pt2.ph_count();
        let mut max_end_vpn = VirtPageNum(0);
        for i in 0..ph_count {
            let ph = elf.program_header(i).unwrap();
            if ph.get_type().unwrap() == xmas_elf::program::Type::Load {
                let start_va: VirtAddr = (ph.virtual_addr() as usize).into();
                let end_va: VirtAddr = ((ph.virtual_addr() + ph.mem_size()) as usize).into();
                let mut map_perm = MapPermission::U;
                let ph_flags = ph.flags();
                if ph_flags.is_read() { map_perm |= MapPermission::R; }
                if ph_flags.is_write() { map_perm |= MapPermission::W; }
                if ph_flags.is_execute() { map_perm |= MapPermission::X; }
                let map_area = MapArea::new(
                    start_va,
                    end_va,
                    MapType::Framed,
                    map_perm,
                );
                max_end_vpn = map_area.vpn_range.get_end();
                memory_set.push(
                    map_area,
                    Some(&elf.input[ph.offset() as usize..(ph.offset() + ph.file_size()) as usize])
                );
            }
        }
        // map user stack with U flags
        let max_end_va: VirtAddr = max_end_vpn.into();
        let mut user_stack_bottom: usize = max_end_va.into();

        // guard page
        user_stack_bottom += PAGE_SIZE;
        let user_stack_top = user_stack_bottom + USER_STACK_SIZE;
        memory_set.push(MapArea::new(
            user_stack_bottom.into(),
            user_stack_top.into(),
            MapType::Framed,
            MapPermission::R | MapPermission::W | MapPermission::U,
        ), None);
        // map TrapContext
        memory_set.push(MapArea::new(
            TRAP_CONTEXT.into(),
            TRAMPOLINE.into(),
            MapType::Framed,
            MapPermission::R | MapPermission::W,
        ), None);

        println!("push shared");
        memory_set.push_shared();


        // memory_set.map_context(space_id);
        
        (memory_set, user_stack_top, elf.header.pt2.entry_point() as usize)
    }

    pub fn from_existed_user(user_space: &MemorySet) -> MemorySet {
        let mut memory_set = Self::new_bare();
        // map trampoline
        memory_set.map_trampoline();
        // copy data sections/trap_context/user_stack
        for area in user_space.areas.iter() {
            let new_area = MapArea::from_another(area);
            memory_set.push(new_area, None);
            // copy data from another space
            for vpn in area.vpn_range {
                let src_ppn = user_space.translate(vpn).unwrap().ppn();
                let dst_ppn = memory_set.translate(vpn).unwrap().ppn();
                dst_ppn.get_bytes_array().copy_from_slice(src_ppn.get_bytes_array());
            }
        }
        memory_set
    }
    pub fn activate(&self) {
        let satp = self.page_table.token();
        unsafe {
            satp::write(satp);
            llvm_asm!("sfence.vma" :::: "volatile");
        }
    }
    pub fn translate(&self, vpn: VirtPageNum) -> Option<PageTableEntry> {
        self.page_table.translate(vpn)
    }
    pub fn recycle_data_pages(&mut self) {
        //*self = Self::new_bare();
        self.areas.clear();
    }

    pub fn init_module(&mut self, elf_data: &[u8]) {

        let elf = xmas_elf::ElfFile::new(elf_data).unwrap();
        let map_len = elf_data.len();
        let base = 0x86000000;
        let start = base as usize;
        let end = base + map_len;
        let start_va: VirtAddr = (start).into();
        let end_va: VirtAddr = (end).into();
        let mut map_perm = MapPermission::W | MapPermission::X | MapPermission::R;
        let map_area = MapArea::new(
            start_va,
            end_va,
            MapType::Identical,
            map_perm,
        );
        println!("start_va {:#x?} end_va {:#x?}", start_va, end_va);
        self.push(map_area,Some(elf_data));
    }
    

    pub fn add_lkm(&mut self, elf_data: &[u8], base: usize){
        let elf = xmas_elf::ElfFile::new(elf_data).unwrap();
        let elf_header = elf.header;
        let magic = elf_header.pt1.magic;
        assert_eq!(magic, [0x7f, 0x45, 0x4c, 0x46], "invalid elf!");
        let ph_count = elf_header.pt2.ph_count();
        let mut max_end_vpn = VirtPageNum(0);

        let mut last_end_va = 0 as usize; 
        for i in 0..ph_count {
            let ph = elf.program_header(i).unwrap();
            if ph.get_type().unwrap() == xmas_elf::program::Type::Load {
                if (ph.virtual_addr() + ph.mem_size()) as usize + base <= last_end_va {
                    continue;
                }

                let mut start = (ph.virtual_addr() ) as usize + base;
                let mut end = (ph.virtual_addr() + ph.mem_size()) as usize + base;
                let mut size = 0 as usize;

                if start < last_end_va &&  end > last_end_va {
                    let diff = last_end_va - start;
                    // println!("diff :{:#x?}", diff);
                    assert!(diff < PAGE_SIZE);
                    start = last_end_va;
                    end = (ph.virtual_addr() + ph.mem_size()) as usize + base - diff;
                }
                
                let start_va: VirtAddr = (start).into();
                let end_va: VirtAddr = (end).into();

                let mut map_perm = MapPermission::W | MapPermission::X | MapPermission::R;
                let ph_flags = ph.flags();

                let map_area = MapArea::new(
                    start_va,
                    end_va,
                    MapType::Identical,
                    map_perm,
                );
                max_end_vpn = map_area.vpn_range.get_end();
                self.push(
                    map_area,
                    Some(&elf.input[ph.offset() as usize..(ph.offset() + ph.file_size()) as usize])
                );

                if ((ph.virtual_addr() + ph.mem_size()) as usize + base) % 0x1000 == 0 {
                    last_end_va = (ph.virtual_addr() + ph.mem_size()) as usize + base;
                }else{
                    last_end_va = (((ph.virtual_addr() + ph.mem_size()) as usize + base)/0x1000) * 0x1000 + 0x1000;
                }

            }
        }
        // map user stack with U flags
        let max_end_va: VirtAddr = max_end_vpn.into();
        let mut user_stack_bottom: usize = max_end_va.into();
        // guard page
        user_stack_bottom += PAGE_SIZE;
        let user_stack_top = user_stack_bottom + USER_STACK_SIZE;
        self.push(MapArea::new(
            user_stack_bottom.into(),
            user_stack_top.into(),
            MapType::Framed,
            MapPermission::R | MapPermission::W | MapPermission::X,
        ), None);

        println!("entry_point :{:#x?}", elf.header.pt2.entry_point() as usize);
    }

}

pub struct MapArea {
    vpn_range: VPNRange,
    data_frames: BTreeMap<VirtPageNum, FrameTracker>,
    map_type: MapType,
    map_perm: MapPermission,
}

impl MapArea {
    pub fn new(
        start_va: VirtAddr,
        end_va: VirtAddr,
        map_type: MapType,
        map_perm: MapPermission
    ) -> Self {
        let start_vpn: VirtPageNum = start_va.floor();
        let end_vpn: VirtPageNum = end_va.ceil();
        Self {
            vpn_range: VPNRange::new(start_vpn, end_vpn),
            data_frames: BTreeMap::new(),
            map_type,
            map_perm,
        }
    }
    pub fn from_another(another: &MapArea) -> Self {
        Self {
            vpn_range: VPNRange::new(another.vpn_range.get_start(), another.vpn_range.get_end()),
            data_frames: BTreeMap::new(),
            map_type: another.map_type,
            map_perm: another.map_perm,
        }
    }
    pub fn map_one(&mut self, page_table: &mut PageTable, vpn: VirtPageNum) {
        let ppn: PhysPageNum;
        match self.map_type {
            MapType::Identical => {
                ppn = PhysPageNum(vpn.0);
            }
            MapType::Framed => {
                let frame = frame_alloc().unwrap();
                ppn = frame.ppn;
                self.data_frames.insert(vpn, frame);
            }
        }
        let pte_flags = PTEFlags::from_bits(self.map_perm.bits).unwrap();
        page_table.map(vpn, ppn, pte_flags);
    }
    pub fn unmap_one(&mut self, page_table: &mut PageTable, vpn: VirtPageNum) {
        match self.map_type {
            MapType::Framed => {
                self.data_frames.remove(&vpn);
            }
            _ => {}
        }
        page_table.unmap(vpn);
    }
    pub fn map(&mut self, page_table: &mut PageTable) {
        for vpn in self.vpn_range {
            self.map_one(page_table, vpn);
        }
    }
    pub fn unmap(&mut self, page_table: &mut PageTable) {
        for vpn in self.vpn_range {
            self.unmap_one(page_table, vpn);
        }
    }
    /// data: start-aligned but maybe with shorter length
    /// assume that all frames were cleared before
    pub fn copy_data(&mut self, page_table: &mut PageTable, data: &[u8]) {
        // assert_eq!(self.map_type, MapType::Framed);
        let mut start: usize = 0;
        let mut current_vpn = self.vpn_range.get_start();
        let len = data.len();
        loop {
            let src = &data[start..len.min(start + PAGE_SIZE)];
            let dst = &mut page_table
                .translate(current_vpn)
                .unwrap()
                .ppn()
                .get_bytes_array()[..src.len()];
            dst.copy_from_slice(src);
            start += PAGE_SIZE;
            if start >= len {
                break;
            }
            current_vpn.step();
        }

    }

    pub fn copy_kernel_data(&mut self, page_table: &mut PageTable, data: &[u8]) {
        let mut start: usize = 0;
        let mut current_vpn = self.vpn_range.get_start();
        let len = data.len();

        use crate::mm::KERNEL_SPACE;
        loop {

            let src = &mut KERNEL_SPACE.lock()
                .translate(current_vpn)
                .unwrap()
                .ppn()
                .get_bytes_array()[..];

            let dst = &mut page_table
                .translate(current_vpn)
                .unwrap()
                .ppn()
                .get_bytes_array()[..src.len()];
            println!("src len:{:#x}", src.len());
            dst.copy_from_slice(src);
            start += PAGE_SIZE;
            if start >= len {
                break;
            }
            current_vpn.step();
        }

        println!("copy kernel data done");
    }

    pub fn copy(&mut self, page_table: &mut PageTable, data: &[u8]) {

        if data.len() == 0x1a4048 {
            println!("data len :{:#x?}", data.len());
        }
        // assert_eq!(self.map_type, MapType::Framed);
        let mut start: usize = 0;
        let mut current_vpn = self.vpn_range.get_start();
        let len = data.len();
        
        loop {
            let src = &data[start..len.min(start + PAGE_SIZE)];
            let dst = &mut page_table
                .translate(current_vpn)
                .unwrap()
                .ppn()
                .get_bytes_array()[..src.len()];
            dst.copy_from_slice(src);
            start += PAGE_SIZE;
            if start >= len {
                break;
            }
            current_vpn.step();
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum MapType {
    Identical,
    Framed,
}

bitflags! {
    pub struct MapPermission: u8 {
        const R = 1 << 1;
        const W = 1 << 2;
        const X = 1 << 3;
        const U = 1 << 4;
    }
}

#[allow(unused)]
pub fn remap_test() {
    let mut kernel_space = KERNEL_SPACE.lock();
    let mid_text: VirtAddr = ((stext as usize + etext as usize) / 2).into();
    let mid_rodata: VirtAddr = ((srodata as usize + erodata as usize) / 2).into();
    let mid_data: VirtAddr = ((sdata as usize + edata as usize) / 2).into();
    assert_eq!(
        kernel_space.page_table.translate(mid_text.floor()).unwrap().writable(),
        false
    );
    assert_eq!(
        kernel_space.page_table.translate(mid_rodata.floor()).unwrap().writable(),
        false,
    );
    assert_eq!(
        kernel_space.page_table.translate(mid_data.floor()).unwrap().executable(),
        false,
    );
    println!("remap_test passed!");
}



fn neg(u: usize) -> usize {
    (-(u as i64)) as usize
}