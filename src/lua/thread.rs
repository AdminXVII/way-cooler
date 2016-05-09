//! Code for the internal Lua thread which handles all Lua requests.

use std::thread;
use std::fs::{File};
use std::path::Path;
use std::io::Write;

use std::fmt::{Debug, Formatter};
use std::fmt::Result as FmtResult;
use std::borrow::Borrow;
use std::collections::BTreeMap;

use std::sync::{Mutex, RwLock};
use std::sync::mpsc::{channel, Sender, Receiver};

use hlua;
use hlua::{Lua, LuaError, LuaTable, PushGuard};
use hlua::any::AnyLuaValue;

use super::types::*;
use super::funcs;

lazy_static! {
    /// Sends requests to the Lua thread
    static ref SENDER: Mutex<Option<Sender<LuaMessage>>> = Mutex::new(None);

    /// Whether the Lua thread is currently running
    pub static ref RUNNING: RwLock<bool> = RwLock::new(false);
}

const ERR_LOCK_RUNNING: &'static str = "Lua thread: unable to lock RUNNING";
const ERR_LOCK_SENDER: &'static str = "Lua thread: unable to lock SENDER";

/// Struct sent to the Lua query
struct LuaMessage {
    reply: Sender<LuaResponse>,
    query: LuaQuery
}

unsafe impl Send for LuaMessage { }
unsafe impl Sync for LuaMessage { }


impl Debug for LuaMessage {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "LuaMessage({:?})", self.query)
    }
}

// Reexported in lua/mod.rs:11
/// Whether the Lua thread is currently available.
pub fn thread_running() -> bool {
    *RUNNING.read().expect(ERR_LOCK_RUNNING)
}

// Reexported in lua/mod.rs:11
/// Errors which may arise from attempting
/// to sending a message to the Lua thread.
#[derive(Debug)]
pub enum LuaSendError {
    /// The thread crashed, was shut down, or rebooted.
    ThreadClosed,
    /// The thread has not been initialized yet (maybe not used)
    ThreadUninitialized,
    /// The sender had an issue, most likey because the thread panicked.
    /// Following the `Sender` API, the original value sent is returned.
    Sender(LuaQuery)
}

// Reexported in lua/mod.rs:11
/// Attemps to send a LuaQuery to the Lua thread.
pub fn send(query: LuaQuery) -> Result<Receiver<LuaResponse>, LuaSendError> {
    if !thread_running() {
        return Err(LuaSendError::ThreadClosed);
    }
    let thread_sender: Sender<LuaMessage>;
    {
        let maybe_sender = SENDER.lock().expect(ERR_LOCK_SENDER);
        match *maybe_sender {
            Some(ref real_sender) => {
                // Senders are designed to be cloneable
                thread_sender = real_sender.clone();
            },
            // If the sender doesn't exist yet, the thread doesn't either
            None => {
                return Err(LuaSendError::ThreadUninitialized);
            }
        }
    }
    // Create a response channel
    let (response_tx, response_rx) = channel();
    let message = LuaMessage { reply: response_tx, query: query };
    match thread_sender.send(message) {
        Ok(_) => Ok(response_rx),
        Err(e) => Err(LuaSendError::Sender(e.0.query))
    }
}

/// Initialize the Lua thread.
pub fn init() {
    trace!("Initializing...");
    let (tx, receiver) = channel();
    *SENDER.lock().expect(ERR_LOCK_SENDER) = Some(tx);
    let mut lua = Lua::new();
    debug!("Loading Lua libraries...");
    lua.openlibs();
    trace!("Loading way-cooler lua extensions...");
    // We should have some good file handling, read files from /usr by default,
    // but for now we're reading directly from the source.
    lua.execute_from_reader::<(), File>(
        File::open("lib/lua/init.lua")
            .expect("Lua thread unable to find init file")
    ).expect("Lua thread: unable to execute init file");
    trace!("Loading way-cooler libraries...");
    funcs::register_libraries(&mut lua);
    // Only ready after loading libs
    *RUNNING.write().expect(ERR_LOCK_RUNNING) = true;
    debug!("Entering main loop...");
    let handle = thread::Builder::new()
        .name("Lua thread".to_string())
        .spawn(move || { main_loop(receiver, &mut lua) });
}

/// Main loop of the Lua thread:
///
/// ## Loop
/// * Wait for a message from the receiver
/// * Handle message
/// * Send response
fn main_loop(receiver: Receiver<LuaMessage>, lua: &mut Lua) {
    loop {
        trace!("Lua: awaiting request");
        let request = receiver.recv();
        match request {
            Err(e) => {
                error!("Lua thread: unable to receive message: {}", e);
                error!("Lua thread: now panicking!");
                *RUNNING.write().expect(ERR_LOCK_RUNNING) = false;

                panic!("Lua thread: lost contact with host, exiting!");
            }
            Ok(message) => {
                trace!("Handling a request");
                thread_handle_message(message, lua);
            }
        }
    }
}

/// Handle each LuaQuery option sent to the thread
fn thread_handle_message(request: LuaMessage, lua: &mut Lua) {
    match request.query {
        LuaQuery::Terminate => {
            trace!("Received terminate signal");
            *RUNNING.write().expect(ERR_LOCK_RUNNING) = false;

            info!("Lua thread terminating!");
            thread_send(request.reply, LuaResponse::Pong);
            return;
        },

        LuaQuery::Restart => {
            trace!("Received restart signal!");
            error!("Lua thread restart not supported!");

            *RUNNING.write().expect(ERR_LOCK_RUNNING) = false;
            thread_send(request.reply, LuaResponse::Pong);

            panic!("Lua thread: Restart not supported!");
        },

        LuaQuery::Execute(code) => {
            trace!("Received request to execute {}", code);

            match lua.execute::<()>(&code) {
                Err(error) => {
                    warn!("Error executing code: {:?}", error);
                    thread_send(request.reply, LuaResponse::Error(error));
                }
                Ok(_) => {
                    trace!("Code executed okay.");
                    thread_send(request.reply, LuaResponse::Pong);
                }
            }
        },

        LuaQuery::ExecFile(name) => {
            info!("Executing {}", name);

            let path = Path::new(&name);
            let try_file = File::open(path);

            if let Ok(file) = try_file {
                let result = lua.execute_from_reader::<(), File>(file);
                if let Err(err) = result {
                    warn!("Error executing {}!", name);

                    thread_send(request.reply, LuaResponse::Error(err));
                }
                else {
                    trace!("Execution of {} successful.", name);
                    thread_send(request.reply, LuaResponse::Pong);
                }
            }
            else { // Could not open file
                // Unwrap_err is used because we're in the else of let Ok
                let read_error =
                    LuaError::ReadError(try_file.unwrap_err());

                thread_send(request.reply, LuaResponse::Error(read_error));
            }
        },
        LuaQuery::ExecRust(func) => {
            let result = func(lua);
            thread_send(request.reply, LuaResponse::Variable(Some(result)));
        },
        LuaQuery::Ping => {
            thread_send(request.reply, LuaResponse::Pong);
        },
    }
}

fn thread_send(sender: Sender<LuaResponse>, response: LuaResponse) {
    match sender.send(response) {
        Err(err) => {
            match err.0 {
                LuaResponse::Pong => {}, // Those are boring
                _ => {
                    warn!("thread: Someone dropped an important Lua response!");
                }
            }
        }
        Ok(_) => {}
    }
}
