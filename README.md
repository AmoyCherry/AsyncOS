# run 
python ./lkm.py

# doc
[scheduler_doc.md](./scheduler_doc.md)

[kernel_module_doc.md](./kernel_module_doc.md)


# requirements

qemu 5.2 
```shell
# 安装编译所需的依赖包
sudo apt install autoconf automake autotools-dev curl libmpc-dev libmpfr-dev libgmp-dev \
              gawk build-essential bison flex texinfo gperf libtool patchutils bc \
              zlib1g-dev libexpat-dev pkg-config  libglib2.0-dev libpixman-1-dev git tmux python3 python3-pip

# 可能还需安装ninja 过程略

# 下载源码包
wget https://download.qemu.org/qemu-5.2.0.tar.xz

# 编译安装并配置 RISC-V 支持
cd qemu-5.2.0
./configure --target-list=riscv64-softmmu,riscv64-linux-user
make -j$(nproc)

# 请注意，qemu-5.2.0 的父目录可以随着你的实际安装位置灵活调整
export PATH=$PATH:$HOME/qemu-5.2.0/build
export PATH=$PATH:$HOME/qemu-5.2.0/build/riscv64-softmmu
export PATH=$PATH:$HOME/qemu-5.2.0/build/riscv64-linux-user
```



# 运行演示
在4核cpu上 .8个用户进程, 每个用户进程下有多个协程运行演示,每个协程输出自身编号

因为时钟中断的原因,中间输出的信息有点乱.


```c
[rustsbi] RustSBI version 0.2.0-alpha.9
.______       __    __      _______.___________.  _______..______   __
|   _  \     |  |  |  |    /       |           | /       ||   _  \ |  |
|  |_)  |    |  |  |  |   |   (----`---|  |----`|   (----`|  |_)  ||  |
|      /     |  |  |  |    \   \       |  |      \   \    |   _  < |  |
|  |\  \----.|  `--'  |.----)   |      |  |  .----)   |   |  |_)  ||  |
| _| `._____| \______/ |_______/       |__|  |_______/    |______/ |__|

[rustsbi] Implementation: RustSBI-QEMU Version 0.0.2
[rustsbi-dtb] Hart count: cluster0 with 4 cores
[rustsbi] misa: RV64ACDFIMSU
[rustsbi] mideleg: ssoft, stimer, sext (0x222)
[rustsbi] medeleg: ima, ia, la, sa, uecall, ipage, lpage, spage (0xb1a3)
[rustsbi] enter supervisor 0x80200000
last 15772 Physical Frames.
.text [0x80200000, 0x8021b000)
.rodata [0x8021b000, 0x80222000)
.data [0x80222000, 0x80223000)
.bss [0x80223000, 0x84264000)
ekernel  MEMORY_END [0x84264000, 0x88000000)
mapping .text section
mapping .rodata section
mapping .data section
mapping .bss section
mapping physical memory
start: 0x87000000 end: 0x87800000
mapping memory-mapped registers
[kernel] Hello, world!
remap_test passed!
loader list app
/**** APPS ****
1
2
3
4
5
6
7
8
initproc
user_shell
basic_rt
**************/
trying to add user test
[hart 0] Start hart[1]
[hart 0] Start hart[2]
[hart 0] Start hart[3]
[hart 0]Hello
[hart 3]init done satp: 0x8000000000084264
[hart 2]init done satp: 0x8000000000084264
[hart 1]init done satp: 0x8000000000084264
[hart 2]Hello
[hart 1]Hello
[hart 3]Hello
[hart 3]run user task
[hart 2]run user task
[hart 1]run user task
[hart 0]run user task
run_tasks
run_tasks
run_tasks
run_tasks
[user5] Hello world from user mode program!
[user8] Hello world from user mode program!
[user7] Hello world from user mode program!
[user6] Hello world from user mode program!
[user3] Hello world from user mode program!
[user4] Hello world from user mode program!
[user2] Hello world from user mode program!
[user1] Hello world from user mode program!

>>>> will switch_to thread 0 in idle_main!
thread_main-------------
thread_main running, no task: 
>>>> will switch_to thread 0 in idle_main!
thread_main-------------
thread_main running, no task: false       
[hart 
>>>> will switch_to thread 0 in idle_main!
thread_main-------------
thread_main running, no task: false       

>>>> will switch_to thread 0 in idle_main!
thread_main-------------
thread_main running, no task: false       
false
[hart 3] [user7] 666
thread_main running, no task: false       
[hart 3] [user7] 9
thread_main running, no task: false       
[hart 3] [user7] 8
thread_main running, no task: false       
>>>> will switch_to thread 0 in idle_main!
thread_main-------------
thread_main running, no task: false        
3] [user5] 666
thread_main running, no task: false        
[hart 3] [user5] 9
thread_main running, no task: false        
[hart 3] [user5] 8
thread_main running, no task: false        
[hart 3] [user5] 7
thread_main running, no task: false[hart 3] [user6] 666
thread_main running, no task: false
[hart 3] [user6] 9
thread_main running, no task: false
[hart 3] [user6] 8
thread_main running, no task: false
[hart 3] [user6] 7
thread_main running, no task: false[hart 3] [user8] 666
thread_main running, no task: false
[hart 3] [user8] 9
thread_main running, no task: false
[hart 3] [user8] 8
thread_main running, no task: false
[hart 3] [user8]
[hart 3] [user7] 7
thread_main running, no task: false
[hart 3] [user7] 6
thread_main running, no task: false
[hart 3] [user7] 5
thread_main running, no task: false
[hart 3] [user7] 4
thread_main running, no task: false
[hart 3] [user3] 666
thread_main running, no task: false
[hart 3] [user3] 9
thread_main running, no task: false
[hart 3] [user3] 8
thread_main running, no task: false
[hart
[hart 3] [user5] 6
thread_main running, no task: false
[hart 3] [user5] 5
thread_main running, no task: false
[hart 3] [user5] 4
thread_main running, no task: false
[hart 3] [user5] 3
thread_main running, no task: false

[hart 3] [user6] 6
thread_main running, no task: false
[hart 3] [user6] 5
thread_main running, no task: false
[hart 3] [user6] 4
thread_main running, no task: false
[hart 3] [user6] 3
thread_main running, no task: false
7
thread_main running, no task: false
[hart 3] [user8] 6
thread_main running, no task: false
[hart 3] [user8] 5
thread_main running, no task: false
[hart 3] [user8] 4
thread_main running, no task: false
[hart 3] [user8] [hart 3] [user7] 3
thread_main running, no task: false
[hart 3] [user7] 2
thread_main running, no task: false
[hart 3] [user7] 1
thread_main running, no task: false
[hart 3] [user7] 0
thread_main running, no task: true3] [user3] 7
thread_main running, no task: false
[hart 3] [user3] 6
thread_main running, no task: false
[hart 3] [user3] 5
thread_main running, no task: false
[hart 3] [user3] 4
thread_main running, no task: false
[hart 3] [user3] [hart 3] [user5] 2
thread_main running, no task: false
[hart 3] [user5] 1
thread_main running, no task: false
[hart 3] [user5] 0
thread_main running, no task: true
no task
user exit
[hart 3] [user6] 2
thread_main running, no task: false
[hart 0] [user6] 1
thread_main running, no task: false
[hart 0] [user6] 0
thread_main running, no task: true
no task
user exit
3
thread_main running, no task: false
[hart 1] [user8] 2
thread_main running, no task: false
[hart 1] [user8] 1
thread_main running, no task: false
[hart 1] [user8] 0
thread_main running, no task: true
no task
user exit
exit_current_and_run_next schedule
3
thread_main running, no task: false
exit_current_and_run_next schedule
[hart 3] [user3] 2
thread_main running, no task: false
[hart 3] [user3] 1
thread_main running, no task: exit_current_and_run_next schedule
false
[hart 3] [user3] 0
thread_main running, no task: true

no task
user exit
exit_current_and_run_next schedule
no task
user exit
exit_current_and_run_next schedule
all user process finished!

>>>> will switch_to thread 0 in idle_main!
thread_main-------------

>>>> will switch_to thread thread_main running, no task: 0false in idle_main!

thread_main-------------
[hart 1] [user4] thread_main running, no task: 666false

[hart 0] [user2] thread_main running, no task: 666false

[hart 1] [user4] 9
thread_main running, no task: thread_main running, no task: falsefalse

[hart [hart 10] [user4] 8] [user2]
9thread_main running, no task:
false
thread_main running, no task: [hart false1
] [user4] [hart 07
] [user2] thread_main running, no task: 8false

thread_main running, no task: [hart false1] [user4]
6
[hart thread_main running, no task: 0false] [user2]
7[hart
1] [user4] 5
thread_main running, no task: thread_main running, no task: falsefalse

[hart [hart 10] [user4] ] [user2] 46

thread_main running, no task: thread_main running, no task: falsefalse

[hart [hart 1] [user4] 03] [user2]
5thread_main running, no task:
falsethread_main running, no task:
[hart false
1[hart ] [user4] 02] [user2]
4thread_main running, no task:
falsethread_main running, no task:
false
[hart [hart 1] [user4] 01] [user2]
3
thread_main running, no task: thread_main running, no task: falsefalse

[hart [hart 1] [user4] 00] [user2]
2
thread_main running, no task: thread_main running, no task: true
falseno task

user exit
[hart 0] [user2] 1
thread_main running, no task: false
[hart 0] [user2] 0
thread_main running, no task: exit_current_and_run_next schedule
trueall user process finished!

no task
user exit
exit_current_and_run_next schedule
all user process finished!

>>>> will switch_to thread 0 in idle_main!
thread_main-------------
thread_main running, no task: false
[hart 2] [user1] 666
thread_main running, no task: false
[hart 2] [user1] 9
thread_main running, no task: false
[hart 2] [user1] 8
thread_main running, no task: false
[hart 2] [user1] 7
thread_main running, no task: false
[hart 2] [user1] 6
thread_main running, no task: false
[hart 2] [user1] 5
thread_main running, no task: false
[hart 2] [user1] 4
thread_main running, no task: false
[hart 2] [user1] 3
thread_main running, no task: false
[hart 2] [user1] 2
thread_main running, no task: false
[hart 2] [user1] 1
thread_main running, no task: false
[hart 2] [user1] 0
thread_main running, no task: true
no task
user exit
exit_current_and_run_next schedule
all user process finished!
Rust user shell
>>
```