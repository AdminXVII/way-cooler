//! The module that controls access to the registry.
//!
//! Permissions for clients live here, which can either be `Read` or `Write`
//! (which includes ability to read).
//!
//! A `Permission` is associated with a canonical category name, with the
//! `Client` controlling all permissions for known categories.
//!
//! Note that it's possible for the `Client` to not know about any given
//! category, effectively making those permissions `None` (because it doesn't
//! even know that it exists).
//!
//! Using a client is the only way to access the registry,
//! allowing the underlying registry implementation to be simple.

use std::borrow::Cow;
use std::collections::hash_map::{self, HashMap};
use super::category::Category;
use super::registry::{Registry, RegistryResult};
use super::REGISTRY2;

/// The mapping of category to the permissions the client has for that category.
pub type AccessMapping = HashMap<String, Permissions>;
/// The result of trying to use a `Client` to access a `Category`.
pub type ClientResult<T> = Result<T, ClientErr>;

/// The different ways the `Client` can fail trying to access a `Category`.
#[derive(Clone, Debug)]
pub enum ClientErr {
    /// A Category does not exist (from the `Client`s perspective)
    DoesNotExist(String),
    /// The `Client` has insufficient permissions to do that operation on
    /// the provided category.
    InsufficientPermissions
}

/// The different ways a client can access the data in a `Category`.
///
/// If a permission for a particular `Category` is omitted, the client by
/// definition cannot access the `Category` from its `Client`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Permissions {
    /// The client can read all data associated with a `Category`.
    Read,
    /// The client can read and write to all data associated with a `Category`.
    Write
}

/// The way a client program accesses the categories in the registry.
///
/// Has a mapping of known `Category`s and its associated permissions.
#[derive(Clone, Debug)]
pub struct Client {
    access: AccessMapping
}

impl Client {
    /// Makes a new client, with the given permissions.
    pub fn new(access: AccessMapping) -> Self {
        Client {
            access: access
        }
    }

    /// Gets an iterator to the categories that the client can access
    pub fn categories<'a>(&'a self) -> hash_map::Iter<'a, String, Permissions> {
        self.access.iter()
    }
}
