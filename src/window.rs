//! Facilities to work with i3 and X11 windows.

use std::process::ChildStderr;

use crate::prelude::*;
use crate::shutils::{cmd, i3_cmd, pipe};
use crate::workspace::Workspace;

pub(crate) const WINDOW_PROPERTIES_SECTION: &str = r#"{name, id, type, "class": .window_properties.class, focused, output, sticky, floating, nodes, window, scratchpad_state}"#;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct Window {
    /// The actual XServer window id
    pub(crate) name: Option<String>,
    /// The container id that i3 uses
    pub(crate) id: u64,
    /// The X11 window id!!!!
    pub(crate) window: Option<u64>,
    #[serde(rename = "type")]
    type_: Option<String>,
    pub(crate) nodes: Option<Vec<Window>>,
    pub(crate) focused: bool,
    pub(crate) class: Option<String>,
    floating: String,
    pub(crate) scratchpad_state: Option<String>,
}

impl Window {
    fn is_floating(&self) -> bool {
        self.floating == "user_on"
    }

    pub(crate) fn name_str(&self) -> String {
        format!(
            "[{:15}] {:20} <{}>",
            self.class.as_ref().unwrap_or(&"".to_string()),
            self.name.as_ref().unwrap_or(&"".to_string()),
            self.scratchpad_state
                .as_ref()
                .unwrap_or(&"No Scratchpad".to_string()),
        )
    }

    /// Return true if this window is some container type like root, content, or workspace
    pub(crate) fn is_container(&self) -> bool {
        // Drop windows that aren't containers
        if self.type_.is_none() {
            true
        } else {
            let type_name = self.type_.as_ref().unwrap();
            if type_name == "con" {
                match &self.name {
                    None => true,
                    Some(name) => name == "content",
                }
            } else {
                true
            }
        }
    }

    fn has_children(&self) -> bool {
        self.nodes.as_ref().map_or(false, |nodes| nodes.len() != 0)
    }

    /// Retrieve a list of all the window names that are a apart of this window's nodes.
    fn node_names(&self) -> Vec<String> {
        let mut out_names: Vec<String> = Vec::new();
        self.node_names_(&mut out_names);
        out_names
    }

    /// Recursively collect all child nodes into a single vector
    pub(crate) fn flatten(&self) -> Vec<Window> {
        let mut children: Vec<Window> = Vec::new();
        self.flatten_window(&mut children);
        children
    }

    pub(crate) fn focus_window(&self) -> Result<String> {
        i3_cmd(&[&format!(r#"[con_id="{}"]"#, self.id), "focus"])
    }

    pub(crate) fn kill(&self) -> Result<String> {
        i3_cmd(&[&format!(r#"[con_id="{}"]"#, self.id), "kill"])
    }

    fn flatten_window(&self, children: &mut Vec<Window>) {
        children.push(self.clone());
        if self.has_children() {
            for child in self.nodes.as_ref().unwrap() {
                child.flatten_window(children)
            }
        }
    }

    /// Push this node's name to an accumulator if it exists.
    fn push_name_(&self, v: &mut Vec<String>) {
        match &self.name {
            None => (),
            Some(name) => {
                v.push(name.to_string());
            }
        }
    }

    /// Retrieve a list of all the window names that are a part of this window's nodes.
    fn node_names_(&self, v: &mut Vec<String>) {
        if self.has_children() {
            match &self.nodes {
                Some(nodes) => {
                    for node in nodes {
                        node.node_names_(v);
                    }
                }
                None => self.push_name_(v),
            };
        } else {
            self.push_name_(v)
        };
    }
}

// List all attached (not floating) windows
pub(crate) fn list_attached_windows() -> Vec<Window> {
    let jq_final_cmd = format!("[.[] | {}]", WINDOW_PROPERTIES_SECTION);

    let mut cmds = [
        &mut cmd(&["i3-msg", "-t", "get_tree"]),
        &mut cmd(&["jq", "[recurse(.nodes[])]"]),
        &mut cmd(&["jq", &jq_final_cmd]),
    ];

    let output = pipe(&mut cmds).unwrap();
    serde_json::from_str::<Vec<Window>>(&output).unwrap()
}

/// List all floating windows
pub(crate) fn list_floating_windows() -> Vec<Window> {
    let jq_final_cmd = format!("[.[] | {}]", WINDOW_PROPERTIES_SECTION);

    let mut cmds = [
        &mut cmd(&["i3-msg", "-t", "get_tree"]),
        &mut cmd(&[
            "jq",
            "[[recurse(.nodes[])] | .[].floating_nodes] | add | .[] | recurse(.nodes[])",
        ]),
        &mut cmd(&["jq", "-s", &jq_final_cmd]),
    ];

    let output = pipe(&mut cmds).unwrap();
    if output.is_empty() {
        Vec::<Window>::new()
    } else {
        serde_json::from_str::<Vec<Window>>(&output).unwrap()
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    #[test]
    fn test_output() {
        let jq_final_cmd = format!("[.[] | {}]", WINDOW_PROPERTIES_SECTION);

        let mut cmds = [
            &mut cmd(&["i3-msg", "-t", "get_tree"]),
            &mut cmd(&[
                "jq",
                "[[recurse(.nodes[])] | .[].floating_nodes] | add | .[] | recurse(.nodes[])",
            ]),
            &mut cmd(&["jq", "-s", &jq_final_cmd]),
        ];

        println!("Running final commands: {:?}", &cmds);
        let output = pipe(&mut cmds).unwrap();
        println!("Output: '{}'", &output);
        let floating_windows = if output.is_empty() {
            Vec::<Window>::new()
        } else {
            serde_json::from_str::<Vec<Window>>(&output).unwrap()
        };

        dbg!(floating_windows);
    }

    #[test]
    fn list_ws() {
        dbg!(list_workspaces());
    }

    #[test]
    fn list_ws_map() {
        dbg!(list_workspaces_and_windows());
    }
}

/// Return all windows managed by i3
pub(crate) fn list_windows() -> Vec<Window> {
    let mut all_windows = list_attached_windows();
    all_windows.extend(list_floating_windows());
    all_windows
}

pub(crate) fn get_workspace_window(workspace_name: &str) -> Window {
    list_windows()
        .iter()
        .find(|w| w.name.as_ref().map_or(false, |name| name == workspace_name))
        .unwrap()
        .clone()
}

/// List at X Server windows whose type is workspace."""
pub(crate) fn list_workspace_windows(workspace_name: &str) -> Vec<Window> {
    let workspace_window = get_workspace_window(workspace_name);
    workspace_window.nodes.unwrap_or(vec![])
}

/// List all the workspaces that are managed by i3
pub(crate) fn list_workspaces() -> Vec<Workspace> {
    let mut cmds = [
        &mut cmd(&["i3-msg", "-t", "get_workspaces"]),
        &mut cmd(&["jq", "-r", "[.[] | {name, id}]"]),
    ];

    let output = pipe(&mut cmds).unwrap();
    serde_json::from_str::<Vec<Workspace>>(&output).unwrap()
}

/// Create a mapping from workspace id => Window
pub(crate) fn list_workspaces_and_windows() -> HashMap<String, Vec<Window>> {
    let workspaces = list_workspaces();

    let mut map: HashMap<String, Vec<Window>> = HashMap::new();

    for ws in workspaces {
        map.insert(ws.name.clone(), list_workspace_windows(&ws.name));
    }

    map
}

pub(crate) fn list_workspaces_and_window_names() -> HashMap<String, Vec<String>> {
    let workspaces = list_workspaces();

    let mut map: HashMap<String, Vec<String>> = HashMap::new();
    for ws in workspaces {
        let ws_name: String = ws.name.clone();
        map.insert(ws_name, get_workspace_window(&ws.name).node_names());
    }

    map
}

/// Retrieve the currently focused window
pub(crate) fn get_focused_window() -> Option<Window> {
    let windows = list_windows();
    windows.iter().find(|window| window.focused).cloned()
}
