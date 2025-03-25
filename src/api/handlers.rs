use crate::{ipc, plugin_tools, ui_interface};
// use hbb_common::{self, config::Config};
use hbb_common;
use std::collections::HashMap;
//use sysinfo::{ProcessExt, System, SystemExt};
use hbb_common::sysinfo::System;

pub fn call_handler(action: &str, payload: &serde_json::Value) -> String {
    match action {
        "get_temporary_password" => get_temporary_password(payload),
        "update_temporary_password" => update_temporary_password(payload),
        // spensercai todo
        "create_new_connect" => create_new_connect(payload),
        "get_server_status" => get_server_status(payload),
        "set_custom_server" => set_custom_server(payload),
        "get_connection_status" => get_connection_status(payload),
        "close_connection_by_id" => close_connection_by_id(payload),
        "set_auto_recording" => set_auto_recording(payload),
        "get_auto_recording" => get_auto_recording(payload),
        _ => {
            let resp = get_resp(0, "wrong action", &serde_json::Value::Null);
            return resp;
        }
    }
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
    let data = serde_json::json!({ "temporary_password": passwd, "id": ipc::get_id() });
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
    let data = serde_json::json!({ "temporary_password": passwd, "id": ipc::get_id() });
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
    plugin_tools::kill_connect(format!("--{} {}", connect_type, id).as_str());
    let resp = get_resp(1, "", &serde_json::Value::Null);
    return resp;
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

// spensercai todo
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
    let my_name = payload["my_name"].as_str().unwrap();
    let temp_paswd = payload["temporary_password"].as_str().unwrap();
    let remote_id = ui_interface::handle_relay_id(&passed_id);
    let my_id = ipc::get_id();
    let force_relay = passed_id != remote_id;
    if remote_id == my_id {
        let resp = get_resp(0, "禁止与自己建立连接", &serde_json::Value::Null);
        return resp;
    }
    // 写入config spensercai todo
    hbb_common::config::LocalConfig::set_my_name(my_name);
    crate::ui_interface::set_peer_option(remote_id.clone().into(), "alias".into(), co_name.into());
    hbb_common::config::LocalConfig::set_remote_id(&remote_id);
    ui_interface::new_remote_with_passwd(
        remote_id.to_string(),
        connect_type.to_string(),
        force_relay,
        temp_paswd.to_string(),
    );
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
    let s = System::new_all();
    let target_process_name = "DarkDesk";
    let mut processes = Vec::<serde_json::Value>::new();
    for process in s.processes_by_name(target_process_name) {
        if process.cmd().contains(&"--connect".to_owned()) {
            processes.push(serde_json::json!({
                "pid": process.pid().to_string(),
                "name": process.name(),
                "type": "controller",
                "peer_id": process.cmd()[process.cmd().len() - 2]
            }));
        }
        if process.cmd().contains(&"--cm".to_owned()) {
            processes.push(serde_json::json!({
                "pid": process.pid().to_string(),
                "name": process.name(),
                "type": "controlled"
            }));
        }
    }
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
