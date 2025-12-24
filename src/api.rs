/*
 * @Author: SpenserCai
 * @Date: 2024-11-22 17:08:06
 * @version:
 * @LastEditors: SpenserCai
 * @LastEditTime: 2024-11-22 20:58:02
 * @Description: file content
 */

// spensercai change
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::thread;

use hbb_common::{log, tokio};

pub mod handlers;
mod pool;

// Public wrapper for handlers::call_handler
pub fn call_handler(action: &str, payload: &serde_json::Value) -> String {
    handlers::call_handler(action, payload)
}

#[tokio::main(flavor = "current_thread")]
pub async fn run() {
    thread::spawn(move || {
        log::info!("{}", "API is running ...");
        let r = TcpListener::bind("127.0.0.1:19876");
        if r.is_err() {
            return;
        }
        let listener = r.unwrap();
        let pool = pool::ThreadPool::new(1);
        for stream in listener.incoming() {
            let stream = stream.unwrap();
            pool.execute(|| handle_connection(stream));
        }
        log::info!("Shutting down...");
    });
}

fn handle_connection(mut stream: TcpStream) {
    // Read incoming data from the client
    let mut buffer = [0; 1024];
    // stream
    //     .read(&mut buffer)
    //     .expect("Failed to read from stream");
    // let received_data = String::from_utf8_lossy(&buffer);
    let bytes_read = match stream.read(&mut buffer) {
        Ok(0) => {
            println!("客户端断开连接");
            return;
        }
        Ok(n) => n,
        Err(e) => {
            eprintln!("读取流失败: {}", e);
            return;
        }
    };
    let received_data = String::from_utf8_lossy(&buffer[..bytes_read]);
    let trimmed_data = received_data.trim_end_matches(char::from(0));
    println!("Received data: {}", trimmed_data);
    let parsed_result = serde_json::from_str::<serde_json::Value>(&trimmed_data);
    let data = match parsed_result {
        Ok(v) => v,
        Err(e) => {
            eprintln!("JSON 解析失败: {}", e);
            let _ = stream.write_all(b"{\"error\": \"invalid json\"}");
            return;
        }
    };

    let action = match data["action"].as_str() {
        Some(a) => a,
        None => {
            eprintln!("请求缺少 'action' 字段");
            let _ = stream.write_all(b"{\"error\": \"missing action\"}");
            return;
        }
    };

    let response = handlers::call_handler(action, &data["payload"]);
    if let Err(e) = stream.write_all(response.as_bytes()) {
        eprintln!("写入响应失败: {}", e);
        return;
    }
        stream.flush().expect("Failed to flush stream");
}
