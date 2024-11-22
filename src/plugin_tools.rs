/*
 * @Author: SpenserCai
 * @Date: 2024-11-22 17:21:57
 * @version: 
 * @LastEditors: SpenserCai
 * @LastEditTime: 2024-11-22 17:41:40
 * @Description: file content
 */
use std::fs;
use std::net::TcpStream;
use std::io::{Write, Result};

// use sysinfo::{ProcessExt, System, SystemExt};
use hbb_common::sysinfo::System;

pub fn kill_connect(key_word: &str) {
    let current_exe = std::env::current_exe().unwrap();
    let current_exe_str = current_exe.to_str().unwrap();
    let current_exe_name = std::path::Path::new(current_exe_str).file_name().unwrap().to_str().unwrap();

    print!("Listing all processes...\n");
    println!("{}",whoami::username());
    let mut system = System::new();
    print!("refreshing process...\n");

    // 刷新系统信息
    system.refresh_processes();
    // 获取并打印所有进程的信息
    for (pid, process) in system.processes() {
        if process.name() == current_exe_name {
            // 把cmd用空格连接
            let mut cmd = process.cmd().join(" ");
            println!("{}: {}", pid, cmd);
            // 如果cmd中包含关键字，就杀掉进程
            if cmd.contains(key_word) {
                print!("Killing process {} with pid {}\n", process.name(), pid);
                process.kill();
            }
        }
        
    }
}