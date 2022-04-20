# 内核可加载模块

# 把模块写入fs.img中

----------easy-fs-fuse\src\main.rs

```rust
let mut host_file = File::open("../tiny_kernel/target/riscv64gc-unknown-none-elf/debug/tiny_kernel").unwrap();
let mut all_data: Vec<u8> = Vec::new();
host_file.read_to_end(&mut all_data).unwrap();
let inode = root_inode.create("tiny_kernel").unwrap();
inode.write_at(0, all_data.as_slice());
```

有时候写入的文件过大需要改变预先设定的fs.img大小 ----------easy-fs-fuse\src\main.rs

```rust
u8表示8bit无符号整数.大小为1B.每个block包含512个u8类型整数.所以每个block是512B 所以4MB = 512B*8192
let block_file = Arc::new(BlockFile(Mutex::new({
    let f = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(format!("{}{}", target_path, "fs.img"))?;
    //set_len以字节为单位. 4MB=512*8192B
    f.set_len(8192 * 512).unwrap();
    f
})));
// 4MiB, at most 4095 files
let efs = EasyFileSystem::create(
    block_file.clone(),
    8192,
    1,
);
```

# 生成可执行文件

大致过程像形成最小化内核那样

需要共享的函数应用#[no_mangle]修饰

并在链接脚本linker.ld中保留整个text段否则会被编译器优化掉

```rust
SECTIONS
{
    . = BASE_ADDRESS;
    .text : {
        KEEP(*(.text*))
        *(.text.entry)
        *(.text .text.*)
    }
.....(略)
```

# 内核加载模块的过程

-------os\src\lkm\mod.rs

```rust
//打开模块文件
let inode = open_file("kernel_module", OpenFlags::RDONLY).unwrap();

//读取文件
let mut v = inode.read_all();

//转成xmas_elf中的elf文件
let elf = ElfFile::new(&v).unwrap();

//从符号表中获取符号的地址
let mut entry = 0 as usize;
for sym  in symbol_table(&elf){
    let name = sym.get_name(&elf);
    let s = "demo_add";
    if name.unwrap() == s{
        println!("name {:?}  value:{:#x?}", name, sym.value());
        entry = sym.value() as usize;
    }
}

//载入内存
let base = 0x87000000 as usize;
KERNEL_SPACE.lock().add_lkm(v.as_slice(), base);

//刷新tlb
KERNEL_SPACE.lock().activate();

//跳转执行
unsafe {
    let demo_add: unsafe extern "C" fn(usize, usize)->usize = transmute(entry + base);
    println!("[LKM] calling demo_add at {:?}", demo_add);
    let x:usize = demo_add(2, 3);
    println!("[LKM] demo_add(2,3) returned {:?}", x);
}
```

# 寻找符号对应的地址

//获得符号表
```rust
fn symbol_table<'a>(elf: &ElfFile<'a>) -> &'a [Entry64] {
    match elf.find_section_by_name(".symtab").unwrap().get_data(&elf).unwrap()
    {
        SymbolTable64(dsym) => dsym,
        _ => panic!("corrupted .symtab"),
    }
}
```
//获取地址

```rust
for sym  in symbol_table(&elf){
    let name = sym.get_name(&elf);
    let s = "demo_add";
    if name.unwrap() == s{
        println!("name {:?}  value:{:#x?}", name, sym.value());
        entry = sym.value() as usize;
    }
}
```

# 内存映射部分
```rust
pub fn add_lkm(&mut self, elf_data: &[u8], base: usize){
    let elf = xmas_elf::ElfFile::new(elf_data).unwrap();
    let elf_header = elf.header;
    let magic = elf_header.pt1.magic;
    assert_eq!(magic, [0x7f, 0x45, 0x4c, 0x46], "invalid elf!");
    let ph_count = elf_header.pt2.ph_count();
    // println!("elf_header.pt2 :{:?}", elf_header.pt2);
    let mut max_end_vpn = VirtPageNum(0);
    let mut last_end_va = 0 as usize; 
    for i in 0..ph_count {
        let ph = elf.program_header(i).unwrap();
        if ph.get_type().unwrap() == xmas_elf::program::Type::Load {
            //有时某ph在上个ph映射时已经被映射过了.因为最少映射一页大小.上个ph的end_va不是4k对齐的话.就要做相应处理.
            if (ph.virtual_addr() + ph.mem_size()) as usize + base <= last_end_va {
                continue;
            }
            let mut start = (ph.virtual_addr() ) as usize + base;
            let mut end = (ph.virtual_addr() + ph.mem_size()) as usize + base;
            let mut size = 0 as usize;
            if start < last_end_va &&  end > last_end_va {
                let diff = last_end_va - start;
                assert!(diff < PAGE_SIZE);
                start = last_end_va;
                end = (ph.virtual_addr() + ph.mem_size()) as usize + base - diff;
            }
            let start_va: VirtAddr = (start).into();
            let end_va: VirtAddr = (end).into();
            //处理完
            
            let mut map_perm = MapPermission::W | MapPermission::X | MapPermission::R;
            let ph_flags = ph.flags();
            let map_area = MapArea::new(
                start_va,
                end_va,
                MapType::Framed,
                map_perm,
            );
            max_end_vpn = map_area.vpn_range.get_end();
            
            //把elf的数据拷贝到内存中.具体是手动查该数组相应区间的虚拟地址所对应的物理地址.然后copy出来,放到要映射的虚拟页所对应的物理页中.
            self.push(
                map_area,
                Some(&elf.input[ph.offset() as usize..(ph.offset() + ph.file_size()) as usize])
            );

            //记录本次映射的地址
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
```

(未做部分)
这里需要内核记录每个模块在内核中的虚拟地址区间.

这样给用户空间映射时,就能通过手动虚拟地址所在的物理地址.并且把其中的数据拷贝出来.再映射到给用户相应的物理页中


# 内核中运行演示

```rust
scheduler init
self.threads len :100
scheduler cpu run
>>>> idle_main
Tid : 0
>>>> will switch_to thread 0 in idle_main!
thread_main-------------------
99
....
0
thread 0 exited, exit code = 0
<<<< switch_back to idle in idle_main!
```