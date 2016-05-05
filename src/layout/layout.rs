//! Main module to handle the layout.
//! This is where the i3specific code is.

 pub mod layout {
    use super::super::container::{Container, Handle};
    use super::super::tree::Node;
    use super::super::super::rustwlc::handle::{WlcView, WlcOutput};

    use std::sync::Mutex;

    lazy_static! {
        static ref ROOT: Mutex<Node> = {
            Mutex::new(Node::new(Container::new_root()))
        };

        static ref CURRENT_WORKSPACE: Mutex<u32> = Mutex::new(0);
    }

    pub fn add_output(wlc_output: WlcOutput) {
        let output = Container::new_output(wlc_output);
        ROOT.lock().unwrap().new_child(output);
        add_workspace();
        add_workspace();
    }

     pub fn add_workspace() {
         let workspace_count = (*ROOT.lock().unwrap()).get_children().len();
         let workspace = Container::new_workspace((workspace_count + 1).to_string());
         (*ROOT.lock().unwrap()).get_children_mut()[0].new_child(workspace);
    }

    pub fn add_view(wlc_view: WlcView) {
        let current_workspace = CURRENT_WORKSPACE.lock().unwrap();
        let mut root = ROOT.lock().unwrap();
        let mut workspace = (*root).get_children_mut()[0].get_children_mut().get_mut(*current_workspace as usize).unwrap();
        workspace.new_child(Container::new_view(wlc_view));
    }

     pub fn remove_view(wlc_view: &WlcView) {
         let mut root = ROOT.lock().unwrap();
         if let Some(view) = root.find_view_by_handle(&wlc_view) {
             let parent = view.get_parent().unwrap();
             parent.remove_child(view);
         }
     }

     pub fn switch_workspace(index: u32) {
         let mut current_workspace = CURRENT_WORKSPACE.lock().unwrap();
         let mut root = ROOT.lock().unwrap();
         let output = &mut (*root).get_children_mut()[0];
         let workspace_children = output.get_children_mut();
         {
             let old_workspace = workspace_children.get_mut(*current_workspace as usize).unwrap();
             for view in old_workspace.get_children_mut() {
                 match view.get_val().get_handle().unwrap() {
                     Handle::View(view) => view.set_mask(0),
                     _ => {},
                 }
             }
         }
         // Assume that the other workspace already exits
         trace!("here");
         let new_workspace = workspace_children.get_mut(index as usize).unwrap();
         trace!("here");
         for view in new_workspace.get_children_mut() {
             match view.get_val().get_handle().unwrap() {
                 Handle::View(view) => view.set_mask(1),
                 _ => {},
             }
         }
         *current_workspace = index;
     }

     fn get_focused_workspace(root: &Node) -> Option<&Node> {
         for output in root.get_children() {
             if output.get_val().is_focused() {
                 for workspace in output.get_children() {
                     if workspace.get_val().is_focused() {
                         return Some(workspace);
                     }
                 }
             }
         }
         None

     }
 }

