use crate::{ipc, ui_interface};
// use hbb_common::{self, config::Config};
use hbb_common::{self, log};
use std::collections::HashMap;
//use sysinfo::{ProcessExt, System, SystemExt};
use hbb_common::sysinfo::System;

pub fn call_handler(action: &str, payload: &serde_json::Value) -> String {
    // If running in Service mode, forward all API calls to GUI/Tray via IPC
    if is_running_in_service() {
        log::info!(
            "Service mode (Session 0): Forwarding API call '{}' to GUI/Tray via IPC",
            action
        );
        return forward_to_user_session(action, payload);
    }

    // Running in user session, handle directly
    log::info!("User session: Handling API call '{}' directly", action);
    match action {
        "get_temporary_password" => get_temporary_password(payload),
        "update_temporary_password" => update_temporary_password(payload),
        "create_new_connect" => create_new_connect(payload),
        "get_server_status" => get_server_status(payload),
        "set_custom_server" => set_custom_server(payload),
        "get_connection_status" => get_connection_status(payload),
        "close_connection_by_id" => close_connection_by_id(payload),
        "set_auto_recording" => set_auto_recording(payload),
        "get_auto_recording" => get_auto_recording(payload),
        "set_permanent_password" => set_permanent_password(payload),
        "get_permanent_password" => get_permanent_password(payload),
        "set_verification_method" => set_verification_method(payload),
        "get_verification_method" => get_verification_method(payload),
        _ => {
            let resp = get_resp(0, "wrong action", &serde_json::Value::Null);
            return resp;
        }
    }
}

// Forward API call to user session via IPC
fn forward_to_user_session(action: &str, payload: &serde_json::Value) -> String {
    use hbb_common::tokio;

    let payload_str = payload.to_string();
    let data = ipc::Data::ApiCall {
        action: action.to_string(),
        payload: payload_str,
    };

    let rt = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => {
            log::error!("Failed to create tokio runtime: {}", e);
            return get_resp(
                -1,
                &format!("Runtime error: {}", e),
                &serde_json::Value::Null,
            );
        }
    };

    rt.block_on(async {
        log::info!("Connecting to IPC server...");
        match ipc::connect(2000, "").await {
            Ok(mut conn) => {
                log::info!("Connected to IPC server, sending API call...");
                if let Err(e) = conn.send(&data).await {
                    log::error!("Failed to send IPC API call: {}", e);
                    return get_resp(
                        -1,
                        &format!("IPC send error: {}", e),
                        &serde_json::Value::Null,
                    );
                }

                log::info!("API call sent, waiting for response (timeout: 5000ms)...");
                // Wait for response
                match conn.next_timeout(5000).await {
                    Ok(Some(ipc::Data::ApiResponse { response })) => {
                        log::info!("Received IPC API response successfully");
                        response
                    }
                    Ok(Some(other)) => {
                        log::error!(
                            "Unexpected IPC response type: {:?}",
                            std::mem::discriminant(&other)
                        );
                        get_resp(-1, "Unexpected response type", &serde_json::Value::Null)
                    }
                    Ok(None) => {
                        log::error!("IPC connection closed without response");
                        get_resp(
                            -1,
                            "Connection closed without response",
                            &serde_json::Value::Null,
                        )
                    }
                    Err(e) => {
                        log::error!("Failed to receive IPC response: {}", e);
                        get_resp(
                            -1,
                            &format!("IPC receive error: {}", e),
                            &serde_json::Value::Null,
                        )
                    }
                }
            }
            Err(e) => {
                log::error!("Failed to connect to IPC server: {}", e);
                get_resp(
                    -1,
                    &format!("IPC connect error: {}. Make sure GUI/Tray is running.", e),
                    &serde_json::Value::Null,
                )
            }
        }
    })
}

// Tool functions ----------------------:
fn get_resp(code: i32, msg: &str, data: &serde_json::Value) -> String {
    let json_str = format!(
        r#"{{
    "code": {code},
    "msg": "{msg}",
    "data": {data}
}}"#
    );
    return json_str;
}

// 返回参数格式错误resp
fn payload_args_format_error() -> String {
    return get_resp(-1, "payload args format error", &serde_json::Value::Null);
}

// Handler functions ----------------------:
fn get_temporary_password(payload: &serde_json::Value) -> String {
    if !check_payload_format(payload, vec!["my_name"]) {
        return payload_args_format_error();
    }
    let my_name = payload["my_name"].as_str().unwrap();
    // spensercai todo
    hbb_common::config::LocalConfig::set_my_name(my_name);
    let passwd = hbb_common::password_security::temporary_password();
    // Use Config::get_id() directly to avoid IPC call within IPC handler
    let data = serde_json::json!({ "temporary_password": passwd, "id": hbb_common::config::Config::get_id() });
    let resp = get_resp(1, "", &data);
    return resp;
}

fn update_temporary_password(payload: &serde_json::Value) -> String {
    if !check_payload_format(payload, vec!["my_name"]) {
        return payload_args_format_error();
    }
    let my_name = payload["my_name"].as_str().unwrap();
    // spensercai todo
    hbb_common::config::LocalConfig::set_my_name(my_name);
    hbb_common::password_security::update_temporary_password();
    let passwd = hbb_common::password_security::temporary_password();
    // Use Config::get_id() directly to avoid IPC call within IPC handler
    let data = serde_json::json!({ "temporary_password": passwd, "id": hbb_common::config::Config::get_id() });
    let resp = get_resp(1, "", &data);
    return resp;
}

// 通过ID关闭连接
fn close_connection_by_id(payload: &serde_json::Value) -> String {
    if !check_payload_format(payload, vec!["id", "connect_type"]) {
        return payload_args_format_error();
    }
    let id = payload["id"].as_str().unwrap();
    let connect_type = payload["connect_type"].as_str().unwrap();

    log::info!("Closing connection: id={}, type={}", id, connect_type);

    // Find the process by ID and connect_type
    let mut s = System::new_all();
    s.refresh_all();

    let keyword = format!("--{} {}", connect_type, id);
    let mut found = false;

    for (pid, process) in s.processes() {
        let process_name = process.name();
        if process_name.to_lowercase().contains("darkdesk") {
            let cmd = process.cmd();
            let cmd_str = cmd.join(" ");

            if cmd_str.contains(&keyword) {
                log::info!("Found connection process: PID={}, cmd={}", pid, cmd_str);

                // Try to kill the process
                if !process.kill() {
                    // If sysinfo kill fails, try Windows API
                    #[cfg(target_os = "windows")]
                    {
                        use std::io::Error;
                        use winapi::um::handleapi::CloseHandle;
                        use winapi::um::processthreadsapi::{OpenProcess, TerminateProcess};
                        use winapi::um::winnt::PROCESS_TERMINATE;

                        unsafe {
                            let handle = OpenProcess(PROCESS_TERMINATE, 0, pid.as_u32());
                            if !handle.is_null() {
                                let result = TerminateProcess(handle, 1);
                                CloseHandle(handle);
                                if result != 0 {
                                    log::info!(
                                        "Successfully killed process {} using Windows API",
                                        pid
                                    );
                                    found = true;
                                } else {
                                    log::error!(
                                        "Failed to terminate process {}: {}",
                                        pid,
                                        Error::last_os_error()
                                    );
                                }
                            } else {
                                log::error!(
                                    "Failed to open process {}: {}",
                                    pid,
                                    Error::last_os_error()
                                );
                            }
                        }
                    }
                    #[cfg(not(target_os = "windows"))]
                    {
                        log::error!("Failed to kill process {}", pid);
                    }
                } else {
                    log::info!("Successfully killed process {} using sysinfo", pid);
                    found = true;
                }
                break;
            }
        }
    }

    if found {
        let resp = get_resp(1, "", &serde_json::Value::Null);
        return resp;
    } else {
        log::warn!(
            "Connection process not found: id={}, type={}",
            id,
            connect_type
        );
        let resp = get_resp(0, "Connection process not found", &serde_json::Value::Null);
        return resp;
    }
}

// check_payload_format,arg:payload,keys
fn check_payload_format(payload: &serde_json::Value, keys: Vec<&str>) -> bool {
    for key in keys {
        if !payload[key].is_string() {
            return false;
        }
    }
    return true;
}

// Helper function to check if running in Service (Session 0)
#[cfg(target_os = "windows")]
fn is_running_in_service() -> bool {
    if let Some(session_id) = crate::platform::get_current_process_session_id() {
        // Session 0 is the service session
        session_id == 0
    } else {
        false
    }
}

#[cfg(not(target_os = "windows"))]
fn is_running_in_service() -> bool {
    // On non-Windows platforms (Mac/Linux), no session separation needed
    // Always return false to handle requests directly
    false
}

// Create new connection (only called in user session now)
fn create_new_connect(payload: &serde_json::Value) -> String {
    if !check_payload_format(
        payload,
        vec!["type", "id", "co_name", "my_name", "temporary_password"],
    ) {
        return payload_args_format_error();
    }
    let connect_type = payload["type"].as_str().unwrap();
    let passed_id = payload["id"].as_str().unwrap();
    let co_name = payload["co_name"].as_str().unwrap();
    let _my_name = payload["my_name"].as_str().unwrap();
    let temp_paswd = payload["temporary_password"].as_str().unwrap();
    let remote_id = ui_interface::handle_relay_id(&passed_id);
    // Use Config::get_id() directly to avoid IPC call within IPC handler
    let my_id = hbb_common::config::Config::get_id();
    let force_relay = passed_id != remote_id;

    if remote_id == my_id {
        let resp = get_resp(0, "禁止与自己建立连接", &serde_json::Value::Null);
        return resp;
    }

    // Set peer options
    crate::ui_interface::set_peer_option(remote_id.to_string(), "alias".into(), co_name.into());
    hbb_common::config::LocalConfig::set_remote_id(&remote_id);

    log::info!(
        "Creating connection: ID={}, Type={}, Force Relay={}",
        remote_id,
        connect_type,
        force_relay
    );

    // Create connection directly (we're in user session)
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        crate::ui_interface::new_remote_with_passwd(
            remote_id.to_string(),
            connect_type.to_string(),
            force_relay,
            temp_paswd.to_string(),
        );
        log::info!("Connection created successfully");
    }

    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        log::error!("Direct connection creation not supported on mobile platforms");
        let resp = get_resp(
            0,
            "Not supported on this platform",
            &serde_json::Value::Null,
        );
        return resp;
    }

    let resp = get_resp(1, "", &serde_json::Value::Null);
    return resp;
}

/*
response:
{
    "code": 1,
    "msg": "",
    "data": {
        "processes":[
            {"name":"聚协云远控","pid":76112,"type":"controlled"},
            {"name":"聚协云远控","pid":76178,"type":"controller"}
        ]
    }
}
*/
fn get_connection_status(_: &serde_json::Value) -> String {
    #[cfg(target_os = "windows")]
    {
        if let Some(session_id) = crate::platform::get_current_process_session_id() {
            log::info!("get_connection_status running in Session {}", session_id);
        }
    }

    // Use System::new_all() to ensure we get all processes
    let mut s = System::new_all();
    s.refresh_all(); // Refresh to get latest process info

    let target_process_name = "DarkDesk";
    let mut processes = Vec::<serde_json::Value>::new();
    let mut total_count = 0;
    let mut all_darkdesk_count = 0;

    log::info!("Scanning for {} processes...", target_process_name);
    log::info!("Total processes in system: {}", s.processes().len());

    // Iterate through ALL processes to find DarkDesk
    for (pid, process) in s.processes() {
        let process_name = process.name();

        // Check if it's a DarkDesk process (case-insensitive)
        if process_name.to_lowercase().contains("darkdesk") {
            all_darkdesk_count += 1;
            let cmd = process.cmd();
            log::info!(
                "Found DarkDesk process: PID={}, name={}, cmd={:?}",
                pid,
                process_name,
                cmd
            );

            if cmd.contains(&"--connect".to_owned()) {
                let mut peer_id = "";
                for i in 0..cmd.len() {
                    if cmd[i] == "--connect" && i + 1 < cmd.len() {
                        peer_id = &cmd[i + 1];
                        break;
                    }
                }
                log::info!("→ Controller process: peer_id={}", peer_id);
                processes.push(serde_json::json!({
                    "pid": pid.to_string(),
                    "name": process_name,
                    "type": "controller",
                    "peer_id": peer_id
                }));
                total_count += 1;
            } else if cmd.contains(&"--cm".to_owned()) {
                log::info!("→ Controlled process (CM)");
                processes.push(serde_json::json!({
                    "pid": pid.to_string(),
                    "name": process_name,
                    "type": "controlled"
                }));
                total_count += 1;
            } else {
                log::debug!("→ Other DarkDesk process (not connection-related)");
            }
        }
    }

    log::info!(
        "Found {} DarkDesk processes total, {} are connections",
        all_darkdesk_count,
        total_count
    );
    let resp = get_resp(1, "", &serde_json::json!({"processes": processes}));
    return resp;
}

fn get_server_status(_: &serde_json::Value) -> String {
    let resp: String;
    let online_status = hbb_common::config::get_online_state();
    println!("{}", online_status);
    if online_status > 0 {
        resp = get_resp(1, "", &serde_json::json!({"status": "READY"}));
    } else if online_status == 0 {
        resp = get_resp(1, "", &serde_json::json!({"status": "CONNECTING"}));
    } else {
        resp = get_resp(1, "", &serde_json::json!({"status": "NOT_READY"}));
    }
    return resp;
}

fn set_custom_server(payload: &serde_json::Value) -> String {
    if !check_payload_format(payload, vec!["id-server", "relay-server", "server-key"]) {
        return payload_args_format_error();
    }
    let rendezvous_server = payload["id-server"].as_str().unwrap();
    let relay_server = payload["relay-server"].as_str().unwrap();
    let server_key = payload["server-key"].as_str().unwrap();
    let mut config_options = HashMap::<String, String>::new();
    config_options.insert(String::from("relay-server"), relay_server.to_string());
    config_options.insert(
        String::from("custom-rendezvous-server"),
        rendezvous_server.to_string(),
    );
    config_options.insert(String::from("key"), server_key.to_string());
    ui_interface::set_options(config_options);
    let resp = get_resp(1, "", &serde_json::Value::Null);
    return resp;
}

// video_save_directory: "/path/to/save"
// allow_auto_record_incoming: "Y" or "N"
// allow_auto_record_outgoing: "Y" or "N"
fn set_auto_recording(payload: &serde_json::Value) -> String {
    if !check_payload_format(
        payload,
        vec![
            "video_save_directory",
            "allow_auto_record_incoming",
            "allow_auto_record_outgoing",
        ],
    ) {
        return payload_args_format_error();
    }
    let video_save_directory = payload["video_save_directory"].as_str().unwrap();
    // allow-auto-record-incoming
    let allow_auto_record_incoming = payload["allow_auto_record_incoming"].as_str().unwrap();
    // allow-auto-record-outgoing
    let allow_auto_record_outgoing = payload["allow_auto_record_outgoing"].as_str().unwrap();

    if video_save_directory.len() != 0 {
        hbb_common::config::LocalConfig::set_option(
            "video-save-directory".to_string(),
            video_save_directory.to_string(),
        );
    }
    // allow-auto-record-incoming MUST be "Y" or "N"
    if allow_auto_record_incoming != "Y" && allow_auto_record_incoming != "N" {
        let resp = get_resp(
            0,
            "allow_auto_record_incoming error,only Y or N",
            &serde_json::Value::Null,
        );
        return resp;
    } else {
        hbb_common::config::Config::set_option(
            "allow-auto-record-incoming".to_string(),
            allow_auto_record_incoming.to_string(),
        );
    }

    // allow-auto-record-outgoing MUST be "Y" or "N"
    if allow_auto_record_outgoing != "Y" && allow_auto_record_outgoing != "N" {
        let resp = get_resp(
            0,
            "allow_auto_record_outgoing error,only Y or N",
            &serde_json::Value::Null,
        );
        return resp;
    } else {
        hbb_common::config::LocalConfig::set_option(
            "allow-auto-record-outgoing".to_string(),
            allow_auto_record_outgoing.to_string(),
        );
    }

    let resp = get_resp(1, "", &serde_json::Value::Null);
    return resp;
}

fn get_auto_recording(_: &serde_json::Value) -> String {
    let auto_recording_in = hbb_common::config::option2bool(
        "allow-auto-record-incoming",
        &hbb_common::config::LocalConfig::get_option("allow-auto-record-incoming"),
    );
    let auto_recording_out = hbb_common::config::option2bool(
        "allow-auto-record-outgoing",
        &hbb_common::config::LocalConfig::get_option("allow-auto-record-outgoing"),
    );
    let video_save_directory: String =
        hbb_common::config::LocalConfig::get_option("video-save-directory");
    // 读取video-save-directory

    let resp = get_resp(
        1,
        "",
        &serde_json::json!({
            "auto_recording_in": auto_recording_in, 
            "auto_recording_out": auto_recording_out,
            "video_save_directory": video_save_directory}),
    );
    return resp;
}

// 设置固定密码
// payload: { "password": "your_password" }
fn set_permanent_password(payload: &serde_json::Value) -> String {
    if !check_payload_format(payload, vec!["password"]) {
        return payload_args_format_error();
    }
    let password = payload["password"].as_str().unwrap();
    hbb_common::config::Config::set_permanent_password(password);
    log::info!("Permanent password has been set");
    get_resp(1, "", &serde_json::Value::Null)
}

// 获取固定密码
fn get_permanent_password(_: &serde_json::Value) -> String {
    let password = hbb_common::config::Config::get_permanent_password();
    let has_password = !password.is_empty();
    // 出于安全考虑，不返回实际密码，只返回是否已设置
    get_resp(
        1,
        "",
        &serde_json::json!({
            "has_password": has_password,
            "password": password
        }),
    )
}

// 设置验证方式
// payload: { "method": "use-permanent-password" | "use-temporary-password" | "use-both-passwords" }
fn set_verification_method(payload: &serde_json::Value) -> String {
    if !check_payload_format(payload, vec!["method"]) {
        return payload_args_format_error();
    }
    let method = payload["method"].as_str().unwrap();

    // 验证方法值是否有效
    let valid_methods = [
        "use-permanent-password",
        "use-temporary-password",
        "use-both-passwords",
    ];
    if !valid_methods.contains(&method) {
        return get_resp(
            0,
            "Invalid method. Valid values: use-permanent-password, use-temporary-password, use-both-passwords",
            &serde_json::Value::Null,
        );
    }

    // 如果选择仅使用固定密码，检查是否已设置固定密码
    if method == "use-permanent-password" {
        let password = hbb_common::config::Config::get_permanent_password();
        if password.is_empty() {
            return get_resp(
                0,
                "Cannot use permanent password only: no permanent password set",
                &serde_json::Value::Null,
            );
        }
    }

    // use-both-passwords 对应空字符串（默认值）
    let config_value = if method == "use-both-passwords" {
        ""
    } else {
        method
    };
    hbb_common::config::Config::set_option(
        "verification-method".to_string(),
        config_value.to_string(),
    );
    log::info!("Verification method set to: {}", method);
    get_resp(1, "", &serde_json::Value::Null)
}

// 获取当前验证方式
fn get_verification_method(_: &serde_json::Value) -> String {
    let method = hbb_common::config::Config::get_option("verification-method");
    let method_name = if method == "use-temporary-password" {
        "use-temporary-password"
    } else if method == "use-permanent-password" {
        "use-permanent-password"
    } else {
        "use-both-passwords"
    };

    get_resp(
        1,
        "",
        &serde_json::json!({
            "method": method_name,
            "temporary_enabled": hbb_common::password_security::temporary_enabled(),
            "permanent_enabled": hbb_common::password_security::permanent_enabled()
        }),
    )
}
