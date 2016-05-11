//! Types used in the registry.

use std::sync::Arc;

use hlua::any::AnyLuaValue;

use super::super::convert::ToTable;

bitflags! {
    /// Access permissions for items in the registry
    pub flags AccessFlags: u8 {
        #[allow(dead_code)]
        const LUA_PRIVATE = 0b00000000,
        const LUA_READ    = 0b00000001,
        #[allow(dead_code)]
        const LUA_WRITE   = 0b00000010,
    }
}

/// Values stored in the registry
#[derive(Debug, PartialEq)]
pub struct RegistryValue {
    flags: AccessFlags,
    object: Arc<AnyLuaValue>
}

impl RegistryValue {
    /// Creates a new RegistryValue
    #[allow(dead_code)]
    pub fn new<T>(flags: AccessFlags, data: T) -> RegistryValue
        where T: ToTable  {
        RegistryValue {
            flags: flags,
            object: Arc::new(data.to_table())
        }
    }

    /// What access the module has to it
    pub fn flags(&self) -> AccessFlags {
        self.flags
    }

    /// Gets the Lua of a registry value
    pub fn get_lua(&self) -> Arc<AnyLuaValue> {
        self.object.clone()
    }

    pub fn set_lua(&mut self, lua: AnyLuaValue) {
        self.object = Arc::new(lua);
    }
}
