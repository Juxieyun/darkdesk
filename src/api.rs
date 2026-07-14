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
    // 设置超时，避免连接无限期挂起
    if let Err(e) = stream.set_read_timeout(Some(std::time::Duration::from_secs(300))) {
        log::error!("Failed to set read timeout: {}", e);
        return;
    }

    // 使用循环保持连接开放，持续处理请求
    loop {
        // Read incoming data from the client
        let mut buffer = [0; 1024];
        let read_result = stream.read(&mut buffer);

        if let Err(e) = read_result {
            log::error!("Failed to read from stream: {}", e);
            break; // 读取错误，结束连接
        }

        let bytes_read = read_result.unwrap();
        if bytes_read == 0 {
            // 客户端关闭连接
            log::debug!("Client closed the connection");
            break;
        }

        let received_data = String::from_utf8_lossy(&buffer[..bytes_read]);
        let trimmed_data = received_data.trim_end_matches(char::from(0));
        log::debug!("Received data: {}", trimmed_data);

        // 检查是否是心跳请求 - 处理可能的空白字符
        let clean_data = trimmed_data.trim();
        if clean_data == "ping" || clean_data == "heartbeat" {
            log::info!("Received heartbeat check");
            if let Err(e) = stream.write("pong".as_bytes()) {
                log::error!("Failed to write heartbeat response: {}", e);
                break;
            }
            if let Err(e) = stream.flush() {
                log::error!("Failed to flush stream after heartbeat: {}", e);
                break;
            }
            continue; // 继续等待下一个请求
        }

        // 如果数据非常短且不是JSON格式，也可能是心跳请求
        if clean_data.len() < 10 && !clean_data.starts_with('{') && !clean_data.starts_with('[') {
            log::info!("Received potential heartbeat: '{}'", clean_data);
            if let Err(e) = stream.write("pong".as_bytes()) {
                log::error!("Failed to write heartbeat response: {}", e);
                break;
            }
            if let Err(e) = stream.flush() {
                log::error!("Failed to flush stream after heartbeat: {}", e);
                break;
            }
            continue; // 继续等待下一个请求
        }

        // 解析JSON并处理错误
        let parsed_result = match serde_json::from_str::<serde_json::Value>(&trimmed_data) {
            Ok(v) => v,
            Err(e) => {
                log::error!("Failed to parse JSON: {}", e);
                let error_response = format!("{{\"error\": \"Invalid JSON format: {}\"}}\n", e);
                if let Err(write_err) = stream.write(error_response.as_bytes()) {
                    log::error!("Failed to write error response: {}", write_err);
                    break;
                }
                if let Err(flush_err) = stream.flush() {
                    log::error!("Failed to flush stream: {}", flush_err);
                    break;
                }
                continue; // 继续等待下一个请求
            }
        };

        // 安全地访问 action 和 payload 字段
        let action = parsed_result.get("action").and_then(|v| v.as_str());
        let payload = parsed_result.get("payload");

        if let Some(action_str) = action {
            log::debug!("parsed_data['action']: {}", action_str);
            if let Some(payload_val) = payload {
                log::debug!("parsed_data['payload']: {}", payload_val);
            } else {
                log::warn!("No payload field found in request");
            }

            let payload_val = payload.unwrap_or(&serde_json::Value::Null);
            let response = format!("{}\n", handlers::call_handler(action_str, payload_val));
            if let Err(e) = stream.write(response.as_bytes()) {
                log::error!("Failed to write to stream: {}", e);
                break;
            }
            if let Err(e) = stream.flush() {
                log::error!("Failed to flush stream: {}", e);
                break;
            }
        } else {
            log::warn!("No action field found in request");
            let error_response = "{\"error\": \"Missing 'action' field\"}\n";
            if let Err(e) = stream.write(error_response.as_bytes()) {
                log::error!("Failed to write error response: {}", e);
                break;
            }
            if let Err(e) = stream.flush() {
                log::error!("Failed to flush stream: {}", e);
                break;
            }
        }
    }

    log::debug!("Connection closed");
}
