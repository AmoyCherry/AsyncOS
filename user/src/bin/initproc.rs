#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

extern crate alloc;

/// Line Feed 换行
const LF: u8 = 0x0au8;
/// Carriage Return 回车
const CR: u8 = 0x0du8;
/// DEL
const DL: u8 = 0x7fu8;
/// Backspace
const BS: u8 = 0x08u8;

use alloc::string::String;
use alloc::vec::Vec;
use user_lib::console::getchar;
use user_lib::{close, dup, exec, fork, open, waitpid, OpenFlags};



#[no_mangle]
pub fn main() -> i32 {
    println!("Rust user shell");
    let mut line: String = String::new();
    print!(">> ");
    loop {
        let c = getchar();
        match c {
            LF | CR => {
                println!("");
                if !line.is_empty() {
                    // 按空格分割 line, 得到的单词装进 args_copy
                    let args: Vec<_> = line.as_str().split(' ').collect();
                    let mut args_copy: Vec<String> = args
                        .iter()
                        .map(|&arg| {
                            let mut string = String::new();
                            string.push_str(arg);
                            string
                        })
                        .collect();
                    // 每个单词末尾插入 \0
                    args_copy.iter_mut().for_each(|string| {
                        string.push('\0');
                    });

    
                    if args_copy.len() == 3 { 
                        println!("exit mmmmmmain");
                        return 0; 
                    }
                    else { 
                        line.clear();
                        print!(">> ");
                        continue; 
                    }

                    // redirect input
                    let mut input = String::new();
                    if let Some((idx, _)) = args_copy
                        .iter()
                        .enumerate()
                        .find(|(_, arg)| arg.as_str() == "<\0")
                    {
                        input = args_copy[idx + 1].clone();
                        args_copy.drain(idx..=idx + 1);
                    }

                    // redirect output
                    let mut output = String::new();
                    if let Some((idx, _)) = args_copy
                        .iter()
                        .enumerate()
                        .find(|(_, arg)| arg.as_str() == ">\0")
                    {
                        output = args_copy[idx + 1].clone();
                        args_copy.drain(idx..=idx + 1);
                    }

                    let mut args_addr: Vec<*const u8> =
                        args_copy.iter().map(|arg| arg.as_ptr()).collect();
                    args_addr.push(0 as *const u8);
                    let pid = fork();
                    if pid == 0 {
                        // input redirection
                        if !input.is_empty() {
                            let input_fd = open(input.as_str(), OpenFlags::RDONLY);
                            if input_fd == -1 {
                                println!("Error when opening file {}", input);
                                return -4;
                            }
                            let input_fd = input_fd as usize;
                            close(0);
                            assert_eq!(dup(input_fd), 0);
                            close(input_fd);
                        }
                        // output redirection
                        if !output.is_empty() {
                            let output_fd =
                                open(output.as_str(), OpenFlags::CREATE | OpenFlags::WRONLY);
                            if output_fd == -1 {
                                println!("Error when opening file {}", output);
                                return -4;
                            }
                            let output_fd = output_fd as usize;
                            close(1);
                            assert_eq!(dup(output_fd), 1);
                            close(output_fd);
                        }
                        // child process
                        if exec(args_copy[0].as_str(), args_addr.as_slice()) == -1 {
                            println!("Error when executing!");
                            return -4;
                        }
                        unreachable!();
                    } else {
                        let mut exit_code: i32 = 0;
                        let exit_pid = waitpid(pid as usize, &mut exit_code);
                        assert_eq!(pid, exit_pid);
                        println!("Shell: Process {} exited with code {}", pid, exit_code);
                    }
                    //line.clear();
                }
                print!(">> ");
            }
            BS | DL => {
                if !line.is_empty() {
                    print!("{}", BS as char);
                    print!(" ");
                    print!("{}", BS as char);
                    line.pop();
                }
            }
            _ => {
                print!("{}", c as char);
                line.push(c as char);
            }
        }
    }

    0
}
