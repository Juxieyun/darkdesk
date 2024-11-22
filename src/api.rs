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

mod handlers;
mod pool;

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
    stream
        .read(&mut buffer)
        .expect("Failed to read from stream");
    let received_data = String::from_utf8_lossy(&buffer);
    let trimmed_data = received_data.trim_end_matches(char::from(0));
    println!("Received data: {}", trimmed_data);

    let parsed_result = serde_json::from_str::<serde_json::Value>(&trimmed_data);
    let data = match parsed_result {
        Ok(v) => v,
        Err(_) => serde_json::Value::Null,
    };
    println!("parsed_data['action']: {}", data["action"]);
    println!("parsed_data['payload']: {}", data["payload"]);

    let response = handlers::call_handler(data["action"].as_str().unwrap(), &data["payload"]);
    stream
        .write(response.as_bytes())
        .expect("Failed to write to stream");
    stream.flush().expect("Failed to flush stream");
}
