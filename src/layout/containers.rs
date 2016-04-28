//! Layout handling

// remove
#![allow(unused)]

use rustwlc::handle::{WlcView, WlcOutput};
use rustwlc::types::VIEW_MAXIMIZED;
use std::rc::{Rc, Weak};
use std::fmt::{Debug, Formatter};
use std::fmt::Result as FmtResult;
use std::cell::RefCell;

pub type Node = Rc<RefCell<Container>>;

#[derive(Debug, Clone)]
enum Handle {
    View(WlcView),
    Output(WlcOutput)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerType {
    /// Root container, only one exists 
    Root,
    /// WlcOutput/Monitor
    Output,
    /// A workspace 
    Workspace,
    /// A Container, houses views and other containers
    Container,
    /// A view (window)
    View
}

/// Layout mode for a container
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layout {
    None,
    Horizontal,
    Vertical,
    Stacked,
    Tabbed,
    Floating
}

#[derive(Clone)]
pub struct Container {
    handle: Option<Handle>,
    parent: Option<Weak<RefCell<Container>>>,
    children: Vec<Node>,
    container_type: ContainerType,
    layout: Layout,
    width: u32,
    height: u32,
    x: i64,
    y: i64,
    visible: bool,
    is_focused: bool,
    is_floating: bool,
}

/// Like i3, everything (workspaces, containers, views) are containable.
impl Container {
    
    /// Makes the root container. There should be only one of these
    /// Does not ensure that this is the only root container
    // NOTE Need to find a way to ensure there is only one of these things
    // Perhaps set a static global variable
    pub fn new_root() -> Node {
        trace!("Root created");
        Rc::new(RefCell::new(Container {
            handle: None,
            parent: None,
            children: vec!(),
            container_type: ContainerType::Root,
            layout: Layout::None,
            width: 0,
            height: 0,
            x: 0,
            y: 0,
            visible: false,
            is_focused: false,
            is_floating: false
        }))
    }
    
    /// Makes a new workspace container. This should only be called by root
    /// since it will properly initialize the right number and properly put
    /// them in the main tree.
    pub fn new_workspace(root: &mut Node) -> Node {
        if ! root.borrow().is_root() {
            panic!("Only workspaces can be added to the root node");
        }
        let workspace: Node =
            Rc::new(RefCell::new(Container {
                // NOTE Give this an output
                handle: None,
                parent: Some(Rc::downgrade(&root)),
                children: vec!(),
                container_type: ContainerType::Workspace,
                // NOTE Change this to some other default
                layout: Layout::None,
                // NOTE Figure out how to initialize these properly
                width: 0,
                height: 0,
                x: 0,
                y: 0,
                visible: false,
                is_focused: false,
                is_floating: false,
                }));
        root.borrow_mut().add_child(workspace.clone());
        workspace
    }

    /// Makes a new container. These hold views and other containers.
    /// Container hold information about specific parts of the tree in some
    /// workspace and the layout of the views within.
    pub fn new_container(parent_: &mut Node, view: WlcView) -> Node {
        let mut parent = parent_.borrow_mut();
        if parent.is_root() {
            panic!("Container cannot be a direct child of root");
        }
        let container = Rc::new(RefCell::new(Container {
            handle: Some(Handle::View(view)),
            parent: Some(Rc::downgrade(&parent_)),
            children: vec!(),
            container_type: ContainerType::Container,
            // NOTE Get default, either from config or from workspace
            layout: Layout::None,
            // NOTE Get this information from somewhere, or set it later
            width: 0,
            height: 0,
            x: 0,
            y: 0,
            visible: false,
            is_focused: false,
            is_floating: false,
        }));
        parent.add_child(container.clone());
        container
    }

    /// Gets the parent that this container sits in.
    ///
    /// If the container is the root, it returns None
    pub fn get_parent(&self) -> Option<Node> {
        if self.is_root() {
            None
        } else {
            // NOTE Clone has to be done here because we have to store the
            // parent as an option since the `Weak::new` is unstable
            if let Some(parent) = self.parent.clone() {
                parent.upgrade()
            } else {
                None
            }
        }
    }

    pub fn add_child(&mut self, container: Node) {
        if self.get_type() == ContainerType::Workspace 
            && container.borrow().get_type() == ContainerType::Workspace {
            panic!("Only containers can be children of a workspace");
        }
        // NOTE check to make sure we are not adding a duplicate
        self.children.push(container);
    }

    /// Removes this container and all of its children
    pub fn remove_container(&mut self) -> Result<(), &'static str> {
        if self.is_root() {
            panic!("Cannot remove root container");
        }
        if let Some(parent) = self.get_parent() {
            parent.borrow_mut().remove_child(self);
        }
        self.children = vec!();
        Ok(())
    }

    /// Gets the children of this container.
    ///
    /// Views never have children
    pub fn get_children(&self) -> Option<Vec<Node>> {
        if self.get_type() == ContainerType::View {
            None
        }
        else {
            Some(self.children.clone())
        } 
    }

    /// Gets the type of the container
    pub fn get_type(&self) -> ContainerType {
        self.container_type
    }

    /// Returns true if this container is focused.
    pub fn is_focused(&self) -> bool {
        self.is_focused
    }

    /// Removes the child at the specified index
    pub fn remove_child_at(&mut self, index: usize) -> Result<Node, &'static str> {
        Ok(self.children.remove(index))
    }

    /// Removes the given child from this container's children.
    /// If the child is not present, then an error is returned
    pub fn remove_child(&mut self, node: &mut Container) -> Result<Node, &'static str> {
        for (index, child) in self.children.clone().iter().enumerate() {
            if *child.borrow() == *node {
                return Ok(self.children.remove(index));
            }
        }
        return Err("");//&format!("Could not find child {:?} in {:?}", node, self));
    }

    /// Sets this container (and everything in it) to given visibility
    pub fn set_visibility(&mut self, visibility: bool) {
        self.visible = visibility
    }

    /// Gets the visibility of the container
    pub fn get_visibility(&self) -> bool {
        self.visible
    }

    /// Gets the X and Y dimensions of the container
    pub fn get_dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Gets the position of this container on the screen
    pub fn get_position(&self) -> (i64, i64) {
        (self.x, self.y)
    }

    /// Returns true if this container is a parent of the child
    pub fn is_parent_of(&self, child: Node) -> bool {
        if self.is_root() {
            true
        } else {
            unimplemented!();
        }
    }

    /// Returns true if this container is a child is an decedent of the parent
    pub fn is_child_of(&self, parent: Node) -> bool {
        if self.is_root() {
            false
        } else {
            if let Some(my_parent) = self.get_parent() {
                my_parent == parent
            } else {
                false 
            }
        }
    }

    pub fn is_root(&self) -> bool {
        self.get_type() == ContainerType::Root
    }

    /// Finds a parent container with the given type, if there is any
    pub fn get_parent_by_type(&self, container_type: ContainerType) -> Option<Node> {
        let mut container = self.get_parent();
        loop {
            if let Some(parent) = container {
                if parent.borrow().get_type() == container_type {
                    return Some(parent);
                }
                container = parent.borrow().get_parent();
            } else {
                return None;
            }
        }
    }
}

impl PartialEq for Container {
    fn eq(&self, other: &Container) -> bool {
        self.get_type() == other.get_type()
    }
}

impl Eq for Container { }

impl Debug for Container {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f.debug_struct("Containable")
            .field("type", &self.get_type())
            .field("parent", &self.get_parent())
            .field("children", &self.get_children())
            .field("focused", &self.is_focused())
            .finish()
    }
}
