//! It's a tree!

use std::fmt::Debug;
use std::ops::{Deref, DerefMut};
use std::ptr;

use rustwlc::handle::{WlcView, WlcOutput};

use super::container::*;

#[derive(Debug, PartialEq, Eq)]
pub struct Node {
    // We need a mut pointer so we can modify the parent
    parent: *mut Node,
    val: Container,
    children: Vec<Node>
}

impl Node {
    /// Create a new node with the existing value.
    /// For root-style constructors.
    pub fn new(val: Container) -> Node {
        Node {
            parent: ptr::null_mut(),
            val: val,
            children: Vec::new()
        }
    }

    /// Add a new child node to this node, using a value
    pub fn new_child(&mut self, val: Container) -> &mut Node {
        let self_mut = self as *mut Node;
        self.children.push(Node {
            parent: self_mut,
            val: val,
            children: Vec::new()
        });
        let last_ix = self.children.len() -1;
        &mut self.children[last_ix]
    }

    /// Whether this node has a (currently-reachable) parent
    pub fn has_parent(&self) -> bool {
        self.parent.is_null()
    }

    /// Gets the type of container this node holds
    pub fn get_container_type(&self) -> ContainerType {
        self.val.get_type()
    }

    /// Tries to get an ancestor of the requested type
    pub fn get_ancestor_of_type<'a>(&'a self, container_type: ContainerType)
                                -> Option<&'a mut Node> {
        let mut maybe_parent = self.get_parent();
        loop {
            if let Some(parent) = maybe_parent {
                if parent.get_container_type() == container_type {
                    return Some(parent);
                }
                maybe_parent = parent.get_parent();
            }
            else {
                return None;
            }
        }
    }

    /// Gets the parent of this node (if it exists)
    pub fn get_parent(&self) -> Option<&mut Node> {
        if self.parent.is_null() {
            return None;
        }
        unsafe {
            return Some(&mut *self.parent);
        }
    }

    /// Borrow the children of this node.
    pub fn get_children(&self) -> &[Node] {
        &self.children
    }

    /// Mutably borrow the children of this mutable node
    pub fn get_mut_children(&mut self) -> &mut[Node] {
        &mut self.children
    }

    /// Remove a child at the given index
    pub fn remove_child_at(&mut self, index: usize) -> Node {
        let mut child = self.children.remove(index);
        child.parent = ptr::null_mut();
        child
    }

    /// Whether this node is a parent of another node
    pub fn is_parent_of(&self, other: &Node) -> bool {
        self.parent == other.parent as *mut Node
    }

    /// Remove a node from its parent.
    /// This method will mutate the parent if it exists.
    pub fn remove_from_parent(&mut self) -> Option<Node> {
        if self.get_parent().is_none() {
            return None;
        }
        if let Some(index) = self.children.iter().position(|c| c == self) {
            self.parent = ptr::null_mut();
            self.children.remove(index);
        }
        return None;
    }

    /// Removes a child from self
    pub fn remove_child(&mut self, other: &Node) -> Option<Node> {
        if let Some(index) = self.children.iter().position(|c| c == other) {
            let mut child = self.children.remove(index);
            child.parent = ptr::null_mut();
            Some(child)
        }
        else {
            None
        }
    }

    pub fn move_to(mut self, new_parent: &mut Node) {
        self.remove_from_parent();
        self.parent = new_parent as *mut Node;
        new_parent.children.push(self);
    }
}

impl Drop for Node {
    fn drop(&mut self) {
        println!("Dropping a node.");
        let children: &mut Vec<Node> = &mut self.children;
        for mut child in children {
            child.parent = ptr::null_mut();
        }
    }
}

#[cfg(test)]
mod tests {
    use log::LogLevelFilter;
    use env_logger::LogBuilder;

    use super::Node;
    use super::super::container::*;

    /// Nodes can have children added to them
    #[test]
    fn test_add_child() {
        let mut root = Node::new(Container::Root);
        root.new_child(Container::Root);
        root.new_child(Container::Root); // This is okay
        {
            let mut third_child = root.new_child(Container::Root);
            //root.new_child(Root); // Have to wait for 3rd child to drop
        }
        root.new_child(Container::Root); // Now this works
        assert_eq!(root.children.len(), 4);
    }

    /// These operations will for example operate on the parent
    /// under an if let. `remove_from_parent` will not panic if the node
    /// already is parentless, for example.
    #[test]
    fn optional_operations() {
        
    }
}
