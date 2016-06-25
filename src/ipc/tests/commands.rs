//! Tests individual commands via `reply`

use std::collections::HashSet;

use rustc_serialize::json::{Json, ToJson};

use super::*;
use super::super::channel;

use commands::tests::wait_for_commands;
use registry;
use registry::tests;
use registry::tests::wait_for_registry;

const PING: &'static str =     r#"{ "type":"ping" }"#;
const COMMANDS: &'static str = r#"{ "type":"commands" }"#;
const VERSION: &'static str =  r#"{ "type":"version" }"#;

const RUN_COMMAND: &'static str =     r#"{"type":"run", "key":"command" }"#;
const RUN_PANIC_COMMAND: &'static str=r#"{"type":"run","key":"panic_command"}"#;
const RUN_BAD_KEY: &'static str =     r#"{"type":"run", "key":"non_command" }"#;
const RUN_NO_KEY: &'static str =      r#"{"type":"run" }"#;

const GET_OBJECT: &'static str =     r#"{ "type":"get", "key":"point" }"#;
const GET_PROP: &'static str =       r#"{ "type":"get", "key":"get_prop" }"#;
const GET_READONLY: &'static str =   r#"{ "type":"get", "key":"readonly" }"#;
const GET_WRITEONLY: &'static str =  r#"{ "type":"get", "key":"writeonly" }"#;
const GET_PROP_WRITE: &'static str = r#"{ "type":"get", "key":"set_prop" }"#;
const GET_NO_KEY: &'static str =     r#"{ "type":"get" }"#;
const GET_BAD_KEY: &'static str =   r#"{ "type":"get", "key":"novalue" }"#;

const SET_OBJECT: &'static str
    = r#"{ "type":"set", "key":"bool", "value":false }"#;
const SET_PROP: &'static str
    = r#"{ "type":"set", "key":"prop", "value":null }"#;
const SET_READONLY: &'static str
    = r#"{ "type":"set", "key":"readonly", "value":12 }"#;
const SET_WRITEONLY: &'static str
    = r#"{ "type":"set", "key":"writeonly", "value":11 }"#;
const SET_PROP_READ: &'static str
    = r#"{ "type":"set", "key":"prop_read", "value":"asdf" }"#;
const SET_NEW_KEY: &'static str
    = r#"{ "type":"set", "key":"new_key", "value":"value" }"#;
const SET_NO_PERMS: &'static str
    = r#"{ "type":"set", "key":"noperms", "value":"something" }"#;
const SET_NO_KEY: &'static str
    = r#"{ "type":"set", "value":12 }"#;
const SET_NO_VALUE: &'static str
    = r#"{ "type":"set", "key":"foo" }"#;
const SET_BAD_KEY_TYPE: &'static str
    = r#"{ "type":"set", "key":12 }"#;

const EXISTS_OBJECT_RW: &'static str =  r#"{"type":"exists","key":"null" }"#;
const EXISTS_PROP: &'static str =       r#"{"type":"exists","key":"prop" }"#;
const EXISTS_PROP_READ: &'static str =  r#"{"type":"exists","key":"get_prop"}"#;
const EXISTS_PROP_WRITE: &'static str = r#"{"type":"exists","key":"set_prop"}"#;
const EXISTS_OBJECT_READ:&'static str = r#"{"type":"exists","key":"readonly"}"#;
const EXISTS_OBJECT_WRITE:&'static str= r#"{"type":"exists","key":"writeonly"}"#;
const EXISTS_BAD_KEY: &'static str =    r#"{"type":"exists","key": "nope"}"#;

const BAD_NO_REQUEST: &'static str = r#"{ "key":"foo", "value":"bar" }"#;
const BAD_INVALID_REQUEST: &'static str = r#"{ "type":"foo" }"#;
const BAD_SET_NO_KEY: &'static str = r#"{ "type":"set", "value":12 }"#;
const BAD_SET_NO_VALUE: &'static str = r#"{ "type":"set", "key":"f64" }"#;
const BAD_RUN_NO_COMMAND: &'static str = r#"{ "type":"run" }"#;
const BAD_RUN_EXTRA_FIELDS: &'static str =
    r#"{ "type":"run", "key":"command", "foo":"bar" }"#;
const BAD_RUN_U64_COMMAND: &'static str = r#"{ "type":"run", "key":12 }"#;
const BAD_GET_OBJECT_KEY: &'static str = r#"{ "type":"get", "key": 12 }"#;
const BAD_REQUEST_IS_A_NUMBER: &'static str = r#"23"#;

macro_rules! reply {
    ($json:expr) => {
        super::super::command::reply(Json::from_str($json)
                                           .expect("Couldn't parse json"))
            .unwrap_or_else(|j| j)
    }
}

#[test]
fn ping() {
    let reply = reply!(PING);
    assert_eq!(reply, channel::success_json());
}

#[test]
fn commands() {
    let mut reply = reply!(COMMANDS);

    let mut obj = reply.as_object_mut()
        .expect("commands: reply not object");
    let mut value = obj.remove("value")
        .expect("commands: reply: no 'value'");
    let val_arr = value.as_array_mut()
        .expect("commands: reply: 'value' not array");
    let mut set = HashSet::new();
    for val in val_arr {
        set.insert(val.as_string()
                   .expect("commands: reply: 'value' not string array"));
    }
    assert_eq!(set.len(), 7);
    let desired = &["get", "set", "exists", "run", "version", "commands", "ping"];
    for com in desired.iter() {
        assert!(set.contains(com), "Missing command {}", com);
    }
}

#[test]
fn version() {
    let mut reply = reply!(VERSION);
    let version = reply.as_object_mut().expect("version: reply not object")
        .remove("value").expect("version: reply: no 'value'")
        .as_u64().expect("version: reply not u64");

    assert_eq!(version, super::super::VERSION);
}

#[test]
fn run_command() {
    wait_for_commands();
    let reply = reply!(RUN_COMMAND);
    assert_eq!(reply, channel::success_json());
}

// Note: If a command were to panic in way-cooler, the thread servicing that
// IPC connection would shut down. However, reply! is routing around that and
// everything is being run in each test thread. This is more of a general way
// of making sure commands are being run and we are aware of them.

#[test]
#[should_panic(reason = "panic_command panic")]
fn run_panic_command() {
    wait_for_commands();
    let reply = reply!(RUN_PANIC_COMMAND);
}

#[test]
fn command_bad_args() {
    wait_for_commands();
    assert_eq!(reply!(RUN_BAD_KEY), Json::Object(json_object!{
        "type" => "error",
        "reason" => "command not found"
    }));
    assert_eq!(reply!(RUN_NO_KEY), Json::Object(json_object!{
        "type" => "error",
        "reason" => "missing message field",
        "missing" => "key",
        "expected" => "String"
    }));
}

#[test]
fn get_stuff() {
    wait_for_registry();
    assert_eq!(reply!(GET_OBJECT),
               channel::value_json(registry::tests::POINT.to_json()));
    assert_eq!(reply!(GET_PROP),
               channel::value_json(json!(registry::tests::PROP_GET_RESULT)));
    assert_eq!(reply!(GET_READONLY),
               channel::value_json(json!(registry::tests::READONLY)));
    assert_eq!(reply!(GET_WRITEONLY),
               channel::value_json(json!(registry::tests::WRITEONLY)));
    assert_eq!(reply!(GET_PROP_WRITE),
               channel::error_json("cannot get that key".to_string()));
    assert_eq!(reply!(GET_NO_KEY),
               channel::error_expecting_key("key", "String"));
    assert_eq!(reply!(GET_BAD_KEY),
               channel::error_json("key not found".to_string()));
}

#[test]
fn set_stuff() {
    wait_for_registry();
    let error_set = channel::error_json("cannot set that key".to_string());
    assert_eq!(reply!(SET_OBJECT), channel::success_json());
    assert_eq!(reply!(SET_PROP), channel::success_json());
    assert_eq!(reply!(SET_READONLY), error_set);
    assert_eq!(reply!(SET_WRITEONLY), channel::success_json());
    assert_eq!(reply!(SET_PROP_READ), error_set);
    assert_eq!(reply!(SET_NO_PERMS), error_set);
    assert_eq!(reply!(SET_NEW_KEY),
               channel::error_json("key not found".to_string()));
    assert_eq!(reply!(SET_NO_KEY),
               channel::error_expecting_key("key", "String"));
    assert_eq!(reply!(SET_BAD_KEY_TYPE),
               channel::error_expecting_key("key", "String"));
    assert_eq!(reply!(SET_NO_VALUE),
               channel::error_expecting_key("Value", "any"));
}
