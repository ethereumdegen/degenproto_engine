use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// A prop value - can be string, number, bool, or asset/variable reference
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum PropValue {
    Str(String),
    Num(f64),
    Bool(bool),
    Var(String),         // Variable reference
    Asset(String),       // Asset reference - looked up in AssetDefs
    Content(String),     // Content reference - looked up in ContentDefs
    ContentField(String), // Field reference within a ContentList context
}

/// An element in the tree
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Element {
    /// Plain text content
    Text(String),

    /// HTML/JSX node
    Node {
        tag: String,
        #[serde(default)]
        class_name: Option<String>,
        #[serde(default)]
        props: HashMap<String, PropValue>,
        #[serde(default)]
        children: Vec<Box<Element>>,
    },

    /// Reference to a component definition
    ComponentRef {
        component: String,
        #[serde(default)]
        props: HashMap<String, PropValue>,
        #[serde(default)]
        children: Vec<Box<Element>>,
    },

    /// Iterate over a content list
    ContentList {
        source: String,           // Key in ContentDefs (must be a List)
        template: Box<Element>,   // Template using ContentField references
    },
}

/// A reusable component definition/preset
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ComponentDef {
    pub name: String,
    pub tag: String,
    #[serde(default)]
    pub class_name: Option<String>,
    #[serde(default)]
    pub default_props: HashMap<String, PropValue>,
    /// Props that must be provided when using this component
    #[serde(default)]
    pub required_props: Vec<String>,
    #[serde(default)]
    pub children_template: Option<Box<Element>>,
    /// Optional path to a JSX component file (generates an import)
    #[serde(default)]
    pub import_path: Option<String>,
}

/// Collection of component definitions
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ComponentDefs {
    pub components: Vec<ComponentDef>,
}

impl ComponentDefs {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let options = ron::Options::default()
            .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME);
        let defs: ComponentDefs = options.from_str(&content)?;
        Ok(defs)
    }

    pub fn get(&self, name: &str) -> Option<&ComponentDef> {
        self.components.iter().find(|c| c.name == name)
    }
}

/// Asset kind
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum AssetKind {
    Image,
    Youtube,
    Video,
    Audio,
}

/// An asset definition (image, video, etc.)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AssetDef {
    pub name: String,
    pub kind: AssetKind,
    #[serde(default)]
    pub path: Option<String>,  // For images, local files
    #[serde(default)]
    pub url: Option<String>,   // For youtube, external URLs
}

/// Collection of asset definitions
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AssetDefs {
    pub assets: Vec<AssetDef>,
}

impl AssetDefs {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let options = ron::Options::default()
            .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME);
        let defs: AssetDefs = options.from_str(&content)?;
        Ok(defs)
    }

    pub fn get(&self, name: &str) -> Option<&AssetDef> {
        self.assets.iter().find(|a| a.name == name)
    }
}

/// Import definition
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Import {
    pub name: String,
    pub path: String,
    #[serde(default)]
    pub kind: ImportKind,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ImportKind {
    #[default]
    Component,
    Asset,
    Hook,
}

/// A view/page definition
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ViewProto {
    pub name: String,
    #[serde(default)]
    pub imports: Vec<Import>,
    #[serde(default)]
    pub observer: bool,
    pub tree: Box<Element>,
}

impl ViewProto {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let options = ron::Options::default()
            .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME);
        let proto: ViewProto = options.from_str(&content)?;
        Ok(proto)
    }
}

/// A content value - can be a string, a record (key-value map), or a list
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ContentValue {
    Str(String),
    Record(HashMap<String, String>),
    List(Vec<ContentValue>),
}

/// Collection of content definitions
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ContentDefs {
    pub content: HashMap<String, ContentValue>,
}

impl ContentDefs {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let options = ron::Options::default()
            .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME);
        let defs: ContentDefs = options.from_str(&content)?;
        Ok(defs)
    }

    pub fn get(&self, name: &str) -> Option<&ContentValue> {
        self.content.get(name)
    }

    /// Get a string value by name
    pub fn get_str(&self, name: &str) -> Option<&String> {
        match self.content.get(name) {
            Some(ContentValue::Str(s)) => Some(s),
            _ => None,
        }
    }

    /// Get a list value by name
    pub fn get_list(&self, name: &str) -> Option<&Vec<ContentValue>> {
        match self.content.get(name) {
            Some(ContentValue::List(list)) => Some(list),
            _ => None,
        }
    }
}
