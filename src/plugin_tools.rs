/*
 * @Author: SpenserCai
 * @Date: 2024-11-22 17:21:57
 * @version:
 * @LastEditors: SpenserCai
 * @LastEditTime: 2025-01-09 09:25:43
 * @Description: file content
 */
use std::fs;
use std::io::{Result, Write};
use std::net::TcpStream;

// use sysinfo::{ProcessExt, System, SystemExt};
use hbb_common::sysinfo::System;

fn send_data_with_socket(msg: &str) -> Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:9877")?;
    stream.write_all(msg.as_bytes())?;
    Ok(())
}

fn send_data_to_self(msg: &str) -> Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:9876")?;
    stream.write_all(msg.as_bytes())?;
    Ok(())
}

fn get_server_status() -> String {
    let mut online_s_str = "";
    let online_status = hbb_common::config::get_online_state();
    println!("{}", online_status);
    if online_status > 0 {
        online_s_str = "READY";
    } else if online_status == 0 {
        online_s_str = "CONNECTING";
    } else {
        online_s_str = "NOT_READY";
    }
    let json_str = format!(
        r#"{{
    "type": "online_status",
    "status": "{online_s_str}"
}}"#
    );
    return json_str;
}

// 发送发送录屏日至
pub fn send_screen_record_log_data(record_type: &str, target_id: &str) -> Result<()> {
    let id = &get_client_id();
    let json_str = format!(
        r#"{{
    "type": "screen_record",
    "id": "{id}",
    "record_type": "{record_type}",
    "target_id": "{target_id}"
}}"#
    );
    send_data_with_socket(&json_str)
}

pub fn send_file_transfer_log_data(
    transfer_type: &str,
    to_path: &str,
    file_name: &str,
) -> Result<()> {
    // 判断是发送文件还是接受文件，发送文件的计算本地文件的md5,接受文件的计算写入本地文件的md5
    let mut use_file = "";

    if transfer_type == "SEND_FILE" {
        use_file = file_name;
    } else if transfer_type == "RECEIVE_FILE" {
        use_file = to_path;
    }

    let file_md5 = if let Ok(src_file) = fs::read(use_file) {
        format!("{:x}", md5::compute(src_file)).clone()
    } else {
        "".to_string()
    };
    let id = &get_client_id();
    let json_str = format!(
        r#"{{
    "type": "file_transfer",
    "id": "{id}",
    "transfer_type": "{transfer_type}",
    "to_path": "{to_path}",
    "file_name": "{file_name}",
    "file_md5": "{file_md5}"
}}"#
    );
    send_data_with_socket(&json_str)
}

// 通过当前进程参数获取client_id
pub fn get_client_id() -> String {
    let args: Vec<String> = std::env::args().collect();
    let mut client_id = "";
    if args.len() > 1 {
        client_id = &args[2];
    }
    return client_id.to_string();
}

// 构造连接被控端状态信息
fn get_connection_status(client_id: &str, status: &str) -> String {
    // 获取当前时间：格式为2021-08-25 15:00:00.000
    let now = chrono::Local::now();
    let now_str = now.format("%Y-%m-%d %H:%M:%S%.3f").to_string();
    let json_str = format!(
        r#"{{
    "type": "connection_status",
    "client_id": "{client_id}",
    "status": "{status}",
    "time": "{now_str}"
}}"#
    );
    return json_str;
}

pub fn send_online_status() -> Result<()> {
    let json_str = get_server_status();
    send_data_with_socket(&json_str)
}

pub fn send_connection_status(client_id: &str, status: &str) -> Result<()> {
    let json_str = get_connection_status(client_id, status);
    send_data_with_socket(&json_str)
}

pub fn kill_connect(key_word: &str) {
    let current_exe = std::env::current_exe().unwrap();
    let current_exe_str = current_exe.to_str().unwrap();
    let current_exe_name = std::path::Path::new(current_exe_str)
        .file_name()
        .unwrap()
        .to_str()
        .unwrap();

    print!("Listing all processes...\n");
    println!("{}", whoami::username());
    let mut system = System::new();
    print!("refreshing process...\n");

    // 刷新系统信息
    system.refresh_processes();
    // 获取并打印所有进程的信息
    for (pid, process) in system.processes() {
        if process.name() == current_exe_name {
            // 把cmd用空格连接
            let cmd = process.cmd().join(" ");
            println!("{}: {}", pid, cmd);
            // 如果cmd中包含关键字，就杀掉进程
            if cmd.contains(key_word) {
                print!("Killing process {} with pid {}\n", process.name(), pid);
                // Try sysinfo kill first
                if !process.kill() {
                    // If sysinfo kill fails (e.g., cross-session), try Windows API
                    #[cfg(target_os = "windows")]
                    {
                        if let Err(e) = kill_process_by_pid(pid.as_u32()) {
                            eprintln!("Failed to kill process {}: {}", pid, e);
                        }
                    }
                }
            }
        }
    }
}

#[cfg(target_os = "windows")]
fn kill_process_by_pid(pid: u32) -> Result<()> {
    use std::io::Error;
    use winapi::um::handleapi::CloseHandle;
    use winapi::um::processthreadsapi::{OpenProcess, TerminateProcess};
    use winapi::um::winnt::PROCESS_TERMINATE;

    unsafe {
        let handle = OpenProcess(PROCESS_TERMINATE, 0, pid);
        if handle.is_null() {
            return Err(Error::last_os_error().into());
        }

        let result = TerminateProcess(handle, 1);
        CloseHandle(handle);

        if result == 0 {
            return Err(Error::last_os_error().into());
        }
    }

    Ok(())
}

pub fn send_close_file_transfer(id: &str) -> Result<()> {
    // message = '{"action":"close_connection_by_id", "payload":{"id":"122656574","connect_type":"file-transfer"}}'
    let json_str = format!(
        r#"{{
    "action": "close_connection_by_id",
    "payload": {{
        "id": "{id}",
        "connect_type": "file-transfer"
    }}
}}"#
    );
    send_data_to_self(&json_str)
}

pub fn send_reboot_client_with_admin() -> Result<()> {
    // message = '{"action":"reboot_client", "payload":{"id":"122656574"}}'
    let json_str = format!(
        r#"{{
    "action": "reboot_client_with_admin",
    "payload": {{
    }}
}}"#
    );
    send_data_to_self(&json_str)
}
