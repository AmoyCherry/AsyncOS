# 每个核上运行线程的抽象

```rust
pub fn init() {
    // 使用 Fifo Scheduler
    let scheduler = FifoScheduler::new();
    // 新建线程池
    let thread_pool = ThreadPool::new(100, Box::new(scheduler));

    // 新建内核线程 idle ，其入口为 Processor::idle_main
    let idle = Thread::new_box_thread(Processor::idle_main as usize, &CPU as *const Processor as usize);

    // 初始化 CPU
    CPU.init(idle, Box::new(thread_pool));

    // 新建一个thread_main加入线程池
    
    CPU.add_thread({
        let thread = Thread::new_box_thread(thread_mian as usize, 1);
        thread
    });
}

CPU.run();
```

# 每个核内运行thread_main

thread_main包含一个协程执行器

```rust
pub fn thread_main() {
    loop {
        let mut queue = USER_TASK_QUEUE.lock();
        let task = queue.peek_task();
        match task {
            // have any task
            Some(task) => {
                let mywaker = task.clone();
                let waker = waker_ref(&mywaker);
                let mut context = Context::from_waker(&*waker);

                let r = task.reactor.clone();
                let mut r = r.lock();

                if r.is_ready(task.id) {
                    let mut future = task.future.lock();
                    match future.as_mut().poll(&mut context) {
                        Poll::Ready(_) => {
                            // 任务完成
                            r.finish_task(task.id);
                        }
                        Poll::Pending => {
                            r.add_task(task.id);
                        }
                    }
                } else if r.contains_task(task.id) {
                    r.add_task(task.id);
                } else {
                    let mut future = task.future.lock();
                    match future.as_mut().poll(&mut context) {
                        Poll::Ready(_) => {
                            // // 任务完成
                            // println!("task completed");
                        }
                        Poll::Pending => {
                            r.register(task.id);
                        }
                    }
                }
            }
            None => return
        }
    }
}
```