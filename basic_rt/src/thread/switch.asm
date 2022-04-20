.equ XLENB, 8
.macro Store reg, mem
    sd \reg, \mem
.endm
.macro Load reg, mem
    ld \reg, \mem
.endm
    # 入栈，即在当前栈上分配空间保存当前 CPU 状态
    # 预分配空间
    addi  sp, sp, (-XLENB*14)

    # 保持当前栈顶地址到a0
    Store sp, 0(a0)
    # 保持当前线程返回地址
    Store ra, 0*XLENB(sp)

    Store s0, 1*XLENB(sp)
    Store s1, 2*XLENB(sp)
    Store s2, 3*XLENB(sp)
    Store s3, 4*XLENB(sp)
    Store s4, 5*XLENB(sp)
    Store s5, 6*XLENB(sp)
    Store s6, 7*XLENB(sp)
    Store s7, 8*XLENB(sp)
    Store s8, 9*XLENB(sp)
    Store s9, 10*XLENB(sp)
    Store s10, 11*XLENB(sp)

    Store s11, 12*XLENB(sp)

    # Store s11, 1*XLENB(sp)
    # 当前线程状态保存完毕
    # 准备恢复到“要切换到的线程”
    # 读取“要切换到的线程栈顶地址”，并直接换栈
    # 由于函数调用第二个参数保持在a1位置....所以要切换到的线程栈顶位置就在a1
    Load sp, 0(a1)
    # sp(1) = s[1] 保持sstatus
    Load s11, 1*XLENB(sp)
    
    Load ra, 0*XLENB(sp)
    Load s0, 1*XLENB(sp)
    Load s1, 2*XLENB(sp)
    Load s2, 3*XLENB(sp)
    Load s3, 4*XLENB(sp)
    Load s4, 5*XLENB(sp)
    Load s5, 6*XLENB(sp)
    Load s6, 7*XLENB(sp)
    Load s7, 8*XLENB(sp)
    Load s8, 9*XLENB(sp)
    Load s9, 10*XLENB(sp)
    Load s10, 11*XLENB(sp)
    Load s11, 12*XLENB(sp)
    mv a0, s0
    # 各寄存器均被恢复，恢复过程结束
    # “要切换到的线程” 变成了 “当前线程”
    # 出栈，即在当前栈上回收用来保存线程状态的内存
    addi sp, sp, (XLENB*14)

    # 将“当前线程的栈顶地址”修改为 0
    # 这并不会修改当前的栈
    # 事实上这个值只有当对应的线程暂停（sleep）时才有效
    # 防止别人企图 switch 到它，把它的栈进行修改
    Store zero, 0(a1)
    ret