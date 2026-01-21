use crate::view_proto::{AssetDefs, AssetKind, ComponentDefs, Element, PropValue, ViewProto};
use std::collections::{HashMap, HashSet};

pub struct ViewJsx {
    pub proto: ViewProto,
    pub component_defs: ComponentDefs,
    pub asset_defs: AssetDefs,
}

impl ViewJsx {
    pub fn new(proto: ViewProto, component_defs: ComponentDefs, asset_defs: AssetDefs) -> Self {
        Self { proto, component_defs, asset_defs }
    }

    pub fn to_string(&self) -> String {
        let mut output = String::new();

        // Collect all asset references used in the tree
        let used_assets = self.collect_asset_refs(&self.proto.tree);

        // React import
        output.push_str("import React from 'react';\n");

        // Observer import if needed
        if self.proto.observer {
            output.push_str("import { observer } from \"mobx-react\";\n");
        }

        output.push('\n');

        // Auto-generate imports for image assets
        for asset_name in &used_assets {
            if let Some(asset) = self.asset_defs.get(asset_name) {
                if let AssetKind::Image = asset.kind {
                    if let Some(path) = &asset.path {
                        output.push_str(&format!("import {} from '{}';\n", asset_name, path));
                    }
                }
            }
        }

        // Component imports from proto (for things like PageSection)
        for import in &self.proto.imports {
            output.push_str(&format!("import {} from '{}';\n", import.name, import.path));
        }

        output.push('\n');

        // Function component
        output.push_str(&format!("function {}() {{\n", self.proto.name));
        output.push_str("  return (\n");

        // Render the tree
        let tree_jsx = self.render_element(&self.proto.tree, 4);
        output.push_str(&tree_jsx);

        output.push_str("  );\n");
        output.push_str("}\n\n");

        // Export
        if self.proto.observer {
            output.push_str(&format!("export default observer({});\n", self.proto.name));
        } else {
            output.push_str(&format!("export default {};\n", self.proto.name));
        }

        output
    }

    fn collect_asset_refs(&self, element: &Element) -> HashSet<String> {
        let mut assets = HashSet::new();
        self.collect_asset_refs_recursive(element, &mut assets);
        assets
    }

    fn collect_asset_refs_recursive(&self, element: &Element, assets: &mut HashSet<String>) {
        match element {
            Element::Text(_) => {}
            Element::Node { props, children, .. } => {
                for value in props.values() {
                    if let PropValue::Asset(name) = value {
                        assets.insert(name.clone());
                    }
                }
                for child in children {
                    self.collect_asset_refs_recursive(child, assets);
                }
            }
            Element::ComponentRef { props, children, .. } => {
                for value in props.values() {
                    if let PropValue::Asset(name) = value {
                        assets.insert(name.clone());
                    }
                }
                for child in children {
                    self.collect_asset_refs_recursive(child, assets);
                }
            }
        }
    }

    fn render_element(&self, element: &Element, indent: usize) -> String {
        match element {
            Element::Text(text) => {
                let indent_str = " ".repeat(indent);
                format!("{}{}\n", indent_str, text)
            }

            Element::Node { tag, class_name, props, children } => {
                self.render_node(tag, class_name.as_deref(), props, children, indent)
            }

            Element::ComponentRef { component, props, children } => {
                // Look up the component definition
                if let Some(def) = self.component_defs.get(component) {
                    // Merge default props with provided props
                    let mut merged_props = def.default_props.clone();
                    for (k, v) in props {
                        merged_props.insert(k.clone(), v.clone());
                    }

                    // Add class_name if defined
                    let class_name = def.class_name.as_deref();

                    self.render_node(&def.tag, class_name, &merged_props, children, indent)
                } else {
                    // Unknown component - render as-is (might be an imported React component)
                    self.render_node(component, None, props, children, indent)
                }
            }
        }
    }

    fn render_node(
        &self,
        tag: &str,
        class_name: Option<&str>,
        props: &HashMap<String, PropValue>,
        children: &[Box<Element>],
        indent: usize,
    ) -> String {
        let indent_str = " ".repeat(indent);
        let mut output = String::new();

        // Opening tag
        output.push_str(&format!("{}<{}", indent_str, tag));

        // Add className if present (from component def)
        if let Some(cn) = class_name {
            // Check if props override className
            if !props.contains_key("className") {
                output.push_str(&format!(" className=\"{}\"", cn));
            }
        }

        // Render props
        for (key, value) in props {
            if key == "text" {
                // Special "text" prop becomes children text
                continue;
            }
            let prop_str = self.render_prop(key, value);
            output.push_str(&format!(" {}", prop_str));
        }

        // Check for text prop (used as inner text)
        let text_content = props.get("text").map(|v| self.prop_value_to_string(v));

        let has_children = !children.is_empty() || text_content.is_some();

        if has_children {
            output.push_str(">\n");

            // Render text content if present
            if let Some(text) = text_content {
                output.push_str(&format!("{}{}\n", " ".repeat(indent + 2), text));
            }

            // Render children
            for child in children {
                output.push_str(&self.render_element(child, indent + 2));
            }

            // Closing tag
            output.push_str(&format!("{}</{}>\n", indent_str, tag));
        } else {
            // Self-closing tag
            output.push_str(" />\n");
        }

        output
    }

    fn render_prop(&self, key: &str, value: &PropValue) -> String {
        match value {
            PropValue::Str(s) => {
                format!("{}=\"{}\"", key, s)
            }
            PropValue::Num(n) => {
                format!("{}={{{}}}", key, n)
            }
            PropValue::Bool(b) => {
                if *b {
                    key.to_string()
                } else {
                    format!("{}={{false}}", key)
                }
            }
            PropValue::Var(var_name) => {
                format!("{}={{{}}}", key, var_name)
            }
            PropValue::Asset(asset_name) => {
                // Look up asset to determine how to render
                if let Some(asset) = self.asset_defs.get(asset_name) {
                    match asset.kind {
                        AssetKind::Image => {
                            // Image assets are imported, use variable reference
                            format!("{}={{{}}}", key, asset_name)
                        }
                        AssetKind::Youtube | AssetKind::Video | AssetKind::Audio => {
                            // URL-based assets use the URL directly
                            if let Some(url) = &asset.url {
                                format!("{}=\"{}\"", key, url)
                            } else {
                                format!("{}=\"\"", key)
                            }
                        }
                    }
                } else {
                    // Unknown asset, treat as variable
                    format!("{}={{{}}}", key, asset_name)
                }
            }
        }
    }

    fn prop_value_to_string(&self, value: &PropValue) -> String {
        match value {
            PropValue::Str(s) => s.clone(),
            PropValue::Num(n) => n.to_string(),
            PropValue::Bool(b) => b.to_string(),
            PropValue::Var(var_name) => format!("{{{}}}", var_name),
            PropValue::Asset(asset_name) => {
                if let Some(asset) = self.asset_defs.get(asset_name) {
                    match asset.kind {
                        AssetKind::Image => format!("{{{}}}", asset_name),
                        _ => asset.url.clone().unwrap_or_default(),
                    }
                } else {
                    format!("{{{}}}", asset_name)
                }
            }
        }
    }
}
