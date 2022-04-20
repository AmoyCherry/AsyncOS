use core::mem::zeroed;

#[repr(C)]
#[derive(Clone)]
pub struct Context {
    content_addr: usize // 上下文内容存储的位置
}


impl Context {
    pub unsafe fn null() -> Context {
        Context { content_addr: 0 }
    }

    pub unsafe fn new_thread_context(
        entry: usize,
        arg: usize,
        ustack_top: usize,
    ) -> Context {
        ContextContent::new_thread_content(entry, arg, ustack_top).push_at(ustack_top)
    }

    #[naked]
    #[inline(never)]
    pub unsafe extern "C" fn switch(&mut self, target: &mut Context) {
        llvm_asm!(include_str!("switch.asm") :::: "volatile");
    }
}


use core::fmt::{self, Debug, Formatter};

impl Debug for Context {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("content_addr:{:#x}", self.content_addr))
    }
}





#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ContextContent {
    // 被调用者保存的寄存器
    pub ra: usize,
    pub s: [usize; 12],

}

impl ContextContent {

    fn new_thread_content(entry: usize, arg: usize , ustack_top: usize) -> ContextContent {
        let mut content: ContextContent = ContextContent::default();
        content.ra = entry as usize;
        content.s[0] = arg;
        content
    }

    // 在指定位置 把ContextContent拷贝到栈中 返回包含有该ContextContent的地址的Context
    unsafe fn push_at(self, stack_top: usize) -> Context {
        let ptr = (stack_top as *mut ContextContent).sub(1);
        *ptr = self; // 拷贝 ContextContent
        Context { content_addr: ptr as usize }
    }
}

impl Default for ContextContent {
    fn default() -> Self {
        unsafe { zeroed() }
    }
}



