//! A client to the Wayland compositor. We control their position through tiling
//! and other properties based on what kind of shell they are.

use std::default::Default;
use std::fmt::{self, Display, Formatter};
use std::hash::{Hash, Hasher};

use rlua::{self, Lua, Table, ToLua, UserData, Value};

use common::{class::{self, Class, ClassBuilder},
             object::{Object, Objectable}};

#[derive(Clone, Debug, Hash)]
pub struct ClientState {
    // TODO Fill in
    pub dummy: i32
}

#[derive(Clone)]
pub struct Client<'lua>(Object<'lua>);

impl Default for ClientState {
    fn default() -> Self {
        ClientState { dummy: 0 }
    }
}

impl<'lua> PartialEq for Client<'lua> {
    fn eq(&self, other: &Self) -> bool {
        &*self.state().unwrap() as *const _ == &*other.state().unwrap() as *const _
    }
}

impl<'lua> Eq for Client<'lua> {}

impl<'lua> Hash for Client<'lua> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.state().unwrap().hash(state);
    }
}

/* This is currently unused.
 * TODO: Figure out if this will be needed later. */
impl <'lua> Client<'lua> {
    pub fn new(lua: &'lua Lua, args: Table) -> rlua::Result<Object<'lua>> {
        let class = class::class_setup(lua, "client")?;
        Ok(Client::allocate(lua, class)?.handle_constructor_argument(args)?
                                        .build())
    }
}

impl Display for ClientState {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Client: {:p}", self)
    }
}

impl<'lua> ToLua<'lua> for Client<'lua> {
    fn to_lua(self, lua: &'lua Lua) -> rlua::Result<Value<'lua>> {
        self.0.to_lua(lua)
    }
}

impl UserData for ClientState {}

pub fn init(lua: &Lua) -> rlua::Result<Class> {
    method_setup(lua, Class::builder(lua, "client", None)?)?.save_class("client")?
                                                            .build()
}

fn method_setup<'lua>(lua: &'lua Lua,
                      builder: ClassBuilder<'lua>)
                      -> rlua::Result<ClassBuilder<'lua>> {
    // TODO Do properly
    use super::dummy;
    builder.method("connect_signal".into(), lua.create_function(dummy)?)?
           .method("__call".into(),
               lua.create_function(|lua, args: Table| Client::new(lua, args))?)?
           .method("get".into(), lua.create_function(dummy_table)?)
}

impl_objectable!(Client, ClientState);

fn dummy_table<'lua>(lua: &'lua Lua, _: rlua::Value) -> rlua::Result<Table<'lua>> {
    Ok(lua.create_table()?)
}
