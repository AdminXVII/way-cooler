//! way-cooler registry.

use std::ops::Deref;
use std::cmp::Eq;
use std::fmt::Display;
use std::hash::Hash;
use std::borrow::Borrow;

use hlua::any::AnyLuaValue;
use convert::{ToTable, FromTable, LuaDecoder, ConverterError};

use std::collections::HashMap;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

mod types;
pub use self::types::*; // Export constants too

#[cfg(test)]
mod tests;

pub type RegMap = HashMap<String, RegistryValue>;

lazy_static! {
    /// Registry variable for the registry
    static ref REGISTRY: RwLock<RegMap> =
        RwLock::new(HashMap::new());
}

/// Error types that can happen
#[derive(Debug, PartialEq, Eq)]
pub enum RegistryError {
    /// The value in the registry could not be parsed
    InvalidLua,
    /// The registry key was not found
    KeyNotFound
}

/// Acquires a read lock on the registry.
pub fn read_lock<'a>() -> RwLockReadGuard<'a, RegMap> {
    REGISTRY.read().unwrap()
}

/// Acquires a write lock on the registry.
pub fn write_lock<'a>() -> RwLockWriteGuard<'a, RegMap> {
    REGISTRY.write().unwrap()
}

/// Gets a Lua object from a registry key
pub fn get_lua<K>(name: &K) -> Option<(AccessFlags, Arc<AnyLuaValue>)>
where String: Borrow<K>, K: Hash + Eq + Display {
    trace!("get_lua: {}", *name);
    let ref reg = *read_lock();
    reg.get(name).map(|val| (val.flags(), val.get_lua()))
}

/// Gets an object from the registry, decoding its internal Lua
/// representation.
pub fn get<K, T>(name: &K) -> Result<(AccessFlags, T), RegistryError>
where T: FromTable, String: Borrow<K>, K: Hash + Eq + Display {
    let maybe_lua = get_lua(name);
    if let Some(lua_pair) = maybe_lua {
        let (access, lua_arc) = lua_pair;
        // Ultimately, values must be cloned out of the registry as well
        match T::from_lua_table(lua_arc.deref().clone()) {
            Ok(val) => Ok((access, val)),
            Err(e) => Err(RegistryError::InvalidLua)
        }
    }
    else {
        Err(RegistryError::KeyNotFound)
    }
}

/// Set a key in the registry to a particular value
pub fn set<T: ToTable>(key: String, flags: AccessFlags, val: T) {
    trace!("set: {:?} {}", flags, key);
    let regvalue = RegistryValue::new(flags, val);
    let ref mut write_reg = *write_lock();
    write_reg.insert(key, regvalue);
}

/// Whether this map contains a key
pub fn contains_key<K>(key: &K) -> bool
where String: Borrow<K>, K: Hash + Eq + Display {
    trace!("contains_key: {}", *key);
    let ref read_reg = *read_lock();
    read_reg.contains_key(key)
}
