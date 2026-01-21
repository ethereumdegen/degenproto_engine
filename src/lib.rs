use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

mod router_jsx;
pub use router_jsx::RouterJsx;

mod view_proto;
pub use view_proto::{ViewProto, Import, ImportKind, Element, PropValue, ComponentDef, ComponentDefs, AssetDef, AssetDefs, AssetKind};

mod view_jsx;
pub use view_jsx::ViewJsx;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProtoIndex {
    pub layouts: Vec<Layout>,
    pub routes: Vec<Route>,
    #[serde(default)]
    pub partials: Vec<Partial>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Layout {
    pub name: String,
    pub path: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Route {
    pub name: String,
    pub url: String,
    pub proto: Option<String>,
    pub path: String,
    pub layout: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Partial {
    pub name: String,
    pub path: String,
}

impl ProtoIndex {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let options = ron::Options::default()
            .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME);
        let index: ProtoIndex = options.from_str(&content)?;
        Ok(index)
    }
}
