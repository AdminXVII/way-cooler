//! Tests for the registry

use std::sync::{Arc, Mutex, Condvar};

use rustc_serialize::json::{Json, ToJson};

use registry;
use registry::AccessFlags;

lazy_static! {
    static ref ACCESS_PAIR: Arc<(Mutex<bool>, Condvar)>
        = Arc::new((Mutex::new(false), Condvar::new()));
}

// Constants for use with registry-accessing tests

pub const BOOL: bool = true;
pub const U64: u64 = 42;
pub const I64: i64 = -1;
pub const F64: f64 = 21.5;

pub const TEXT: &'static str = "Hello way-cooler";
pub const POINT: Point = Point { x: 12, y: 12 };

pub const READONLY: &'static str = "read only text";
pub const WRITEONLY: &'static str = "write only text";
pub const NO_PERMS: &'static str = "<look ma no perms>";

pub const PROP_GET_RESULT: &'static str = "get property";

pub fn get_prop() -> Json {
    PROP_GET_RESULT.to_json()
}

pub fn get_panic_prop() -> Json {
    panic!("get_panic_prop panic")
}

pub fn set_prop(_json: Json) {
    println!("set_prop being called!");
}

pub fn set_panic_prop(_json: Json) {
    panic!("set_panic_prop panic")
}

pub fn command() {
    println!("command being run!");
}

pub fn panic_command() {
    panic!("panic_command panic")
}

/// [0, 1, 2]
pub fn u64s() -> Vec<Json> {
    vec![Json::U64(0), Json::U64(1), Json::U64(2)]
}

/// Wait for the registry to initialize
pub fn wait_for_registry() {
    // First star for lazy static, second for Arc
    let &(ref lock, ref cond) = &**ACCESS_PAIR;
    let mut started = lock.lock().expect("Unable to lock ACCESS_PAIR lock!");
    while !*started {
        started = cond.wait(started)
            .expect("Oh boy, I can't wait for the registry to start!");
    }
}

json_convertible! {
    /// Has an x and y field
    #[derive(Debug, Clone, Eq, PartialEq)]
    struct Point {
        x: i32,
        y: i32
    }
}

unsafe impl Sync for Point {}

#[test]
fn add_keys() {
    let values = vec![
        ("bool",  AccessFlags::all(), BOOL.to_json()),
        ("u64",   AccessFlags::all(), U64.to_json()),
        ("i64",   AccessFlags::all(), I64.to_json()),
        ("f64",   AccessFlags::all(), F64.to_json()),
        ("text",  AccessFlags::all(), TEXT.to_json()),
        ("point", AccessFlags::all(), POINT.to_json()),
        ("null",  AccessFlags::all(), Json::Null),
        ("u64s",  AccessFlags::all(), Json::Array(u64s())),

        ("readonly",  AccessFlags::READ(),  READONLY.to_json()),
        ("writeonly", AccessFlags::WRITE(), WRITEONLY.to_json()),
        ("noperms",   AccessFlags::empty(), NO_PERMS.to_json())
    ];

    for (name, flags, json) in values.into_iter() {
        registry::set_json(name.to_string(), flags, json)
            .expect("Unable to initialize objects in registry");
    }

    // Command
    registry::set_command("command".to_string(), Arc::new(command))
        .expect("Unable to initialize command in registry");

    // Panicking command
    registry::set_command("panic_command".to_string(), Arc::new(panic_command))
        .expect("Unable to initialize command in registry");

    // Read/write property
    registry::set_property_field("prop".to_string(),
                                 Some(Arc::new(get_prop)),
                                 Some(Arc::new(set_prop)))
        .expect("Unable to initialize property in registry");

    // Readonly/writeonly properties
    registry::set_property_field("get_prop".to_string(),
                                 Some(Arc::new(get_prop)),
                                 None)
        .expect("Unable to initialize property in registry");
    registry::set_property_field("set_prop".to_string(),
                                 None,
                                 Some(Arc::new(set_prop)))
        .expect("Unable to initialize property in registry");

    // Readonly/writeonly panicking properties
    registry::set_property_field("get_panic_prop".to_string(),
                                 Some(Arc::new(get_panic_prop)),
                                 None)
        .expect("Unable to initialize property in registry");
    registry::set_property_field("set_panic_prop".to_string(),
                                 None,
                                 Some(Arc::new(set_panic_prop)))
        .expect("Unable to initialize property in registry");

    // Allow waiting threads to continue
    let &(ref lock, ref cond) = &**ACCESS_PAIR;
    let mut started = lock.lock().expect("Couldn't unlock threads");
    *started = true;
    cond.notify_all();
}

#[test]
fn contains_keys() {
    wait_for_registry();

    let keys = [
        "bool", "u64", "i64", "f64", "null", "text", "point",
        "u64s", "readonly", "writeonly", "prop", "noperms",
        "get_prop", "set_prop", "command",
        "get_panic_prop", "set_panic_prop", "panic_command",
    ];
    for key in keys.into_iter() {
        assert!(registry::contains_key(key),
                "Could not find key {}", key);
    }
}

#[test]
fn objects_and_keys_equal() {
    wait_for_registry();

    let values = vec![
        ("bool",  AccessFlags::all(), BOOL.to_json()),
        ("u64",   AccessFlags::all(), U64.to_json()),
        ("i64",   AccessFlags::all(), I64.to_json()),
        ("f64",   AccessFlags::all(), F64.to_json()),
        ("text",  AccessFlags::all(), TEXT.to_json()),
        ("point", AccessFlags::all(), POINT.to_json()),
        ("null",  AccessFlags::all(), Json::Null),
        ("u64s",  AccessFlags::all(), Json::Array(u64s())),

        ("readonly",  AccessFlags::READ(),  READONLY.to_json()),
        ("writeonly", AccessFlags::WRITE(), WRITEONLY.to_json()),
        ("noperms",   AccessFlags::empty(), NO_PERMS.to_json())
    ];

    for (name, flags, json) in values.into_iter() {
        let (found_flags, found) = registry::get_json(name)
            .expect(&format!("Unable to get key {}", name));
        assert_eq!(found_flags, flags);
        assert_eq!(*found, json);
    }
}

#[test]
fn key_perms() {
    wait_for_registry();

    let perms = vec![
        ("bool",      AccessFlags::all()),
        ("readonly",  AccessFlags::READ()),
        ("writeonly", AccessFlags::WRITE()),
        ("prop",      AccessFlags::all()),
        //("prop_read", AccessFlags::READ()),
        //("prop_write", AccessFlags::WRITE())
    ];

    for (name, flags) in perms.into_iter() {
        let found_data = registry::get_data(name)
            .expect(&format!("Could not get data for {}", name));

        let (found_flags, _) = found_data.resolve();
        println!("Testing flags for {}", name);
        assert_eq!(found_flags, flags);
    }
}

#[test]
fn run_command() {
    wait_for_registry();
    let command = registry::get_command("command")
        .expect("Command not found");
    command();
}

#[test]
#[should_panic(expected = "command panic")]
fn run_panic_command() {
    wait_for_registry();
    let command = registry::get_command("panic_command")
        .expect("Command not found");
    command();
}

#[test]
fn property_get() {
    wait_for_registry();
    let prop_read = registry::get_data("get_prop")
        .expect("Couldn't get prop_read");

    assert_eq!(*prop_read.resolve().1, PROP_GET_RESULT.to_json());
}

#[test]
#[should_panic(expected="get_panic_prop panic")]
fn panicking_property_get() {
    wait_for_registry();
    let prop_read = registry::get_data("get_panic_prop")
        .expect("Couldn't get prop_read");

    assert_eq!(*prop_read.resolve().1, PROP_GET_RESULT.to_json());
}

#[test]
fn property_set() {
    wait_for_registry();
    assert!(registry::set_json("set_prop".to_string(),
                               AccessFlags::empty(), Json::Null)
            .expect("Unable to set data").is_none());
}

#[test]
#[should_panic(expected="set_panic_prop panic")]
fn panicking_property_set() {
    wait_for_registry();
    assert!(registry::set_json("set_panic_prop".to_string(),
                               AccessFlags::empty(), Json::Null)
            .expect("Unable to set data").is_none());
}
