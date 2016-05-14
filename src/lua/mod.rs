//! Lua functionality

#[macro_use]
mod funcs;
#[cfg(test)]
mod tests;

mod types;
mod thread;
pub use self::types::{LuaQuery, LuaFunc, LuaResponse};

pub use self::thread::{running, send, LuaSendError};

/// Initialize the lua thread
pub fn init() {
    trace!("Initializing...");

    // The Lua thread will start the IPC thread
    // after it initializes.
    ::std::thread::spawn(move || {
        thread::init();
    });
    trace!("Lua initialization finished.");
}

