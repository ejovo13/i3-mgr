/// Facilities for working withn i3 workspaces.
///
use crate::prelude::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct Workspace {
    /// The actual XServer window id
    pub(crate) id: u64,
    pub(crate) name: String,
}
