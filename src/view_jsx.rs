use crate::view_proto::{AssetDefs, AssetKind, ComponentDefs, ContentDefs, ContentValue, Element, PropValue, ViewProto};
use std::collections::{HashMap, HashSet};

pub struct ViewJsx {
    pub proto: ViewProto,
    pub component_defs: ComponentDefs,
    pub asset_defs: AssetDefs,
    pub content_defs: ContentDefs,
}

impl ViewJsx {
    pub fn new(proto: ViewProto, component_defs: ComponentDefs, asset_defs: AssetDefs, content_defs: ContentDefs) -> Self {
        Self { proto, component_defs, asset_defs, content_defs }
    }

    pub fn to_string(&self) -> String {
        let mut output = String::new();

        // Collect all asset references used in the tree
        let used_assets = self.collect_asset_refs(&self.proto.tree);

        // Collect all component references used in the tree
        let used_components = self.collect_component_refs(&self.proto.tree);

        // React import
        output.push_str("import React from 'react';\n");

        // Observer import if needed
        if self.proto.observer {
            output.push_str("import { observer } from \"mobx-react\";\n");
        }

        output.push('\n');

        // Auto-generate imports for image assets (skip external URLs)
        for asset_name in &used_assets {
            if let Some(asset) = self.asset_defs.get(asset_name) {
                if let AssetKind::Image = asset.kind {
                    if let Some(path) = &asset.path {
                        // Don't import external URLs
                        if !path.starts_with("http://") && !path.starts_with("https://") {
                            output.push_str(&format!("import {} from '{}';\n", asset_name, path));
                        }
                    }
                }
            }
        }

        // Auto-generate imports for components with import_path
        for component_name in &used_components {
            if let Some(def) = self.component_defs.get(component_name) {
                if let Some(import_path) = &def.import_path {
                    output.push_str(&format!("import {} from '{}';\n", def.tag, import_path));
                }
            }
        }

        // Manual imports from proto (fallback for anything not in component_defs)
        for import in &self.proto.imports {
            output.push_str(&format!("import {} from '{}';\n", import.name, import.path));
        }

        output.push('\n');

        // Function component
        output.push_str(&format!("function {}() {{\n", self.proto.name));
        output.push_str("  return (\n");

        // Render the tree
        let tree_jsx = self.render_element(&self.proto.tree, 4, None);
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
        self.collect_refs_recursive(element, &mut assets, &mut HashSet::new());
        assets
    }

    fn collect_component_refs(&self, element: &Element) -> HashSet<String> {
        let mut components = HashSet::new();
        self.collect_refs_recursive(element, &mut HashSet::new(), &mut components);
        components
    }

    fn collect_refs_recursive(
        &self,
        element: &Element,
        assets: &mut HashSet<String>,
        components: &mut HashSet<String>,
    ) {
        match element {
            Element::Text(_) => {}
            Element::Node { props, children, .. } => {
                for value in props.values() {
                    if let PropValue::Asset(name) = value {
                        assets.insert(name.clone());
                    }
                }
                for child in children {
                    self.collect_refs_recursive(child, assets, components);
                }
            }
            Element::ComponentRef { component, props, children } => {
                components.insert(component.clone());
                for value in props.values() {
                    if let PropValue::Asset(name) = value {
                        assets.insert(name.clone());
                    }
                }
                for child in children {
                    self.collect_refs_recursive(child, assets, components);
                }
            }
            Element::ContentList { template, .. } => {
                self.collect_refs_recursive(template, assets, components);
            }
        }
    }

    fn render_element(&self, element: &Element, indent: usize, record_ctx: Option<&HashMap<String, String>>) -> String {
        match element {
            Element::Text(text) => {
                let indent_str = " ".repeat(indent);
                format!("{}{}\n", indent_str, text)
            }

            Element::Node { tag, class_name, props, children } => {
                self.render_node(tag, class_name.as_deref(), props, children, indent, record_ctx)
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

                    self.render_node(&def.tag, class_name, &merged_props, children, indent, record_ctx)
                } else {
                    // Unknown component - render as-is (might be an imported React component)
                    self.render_node(component, None, props, children, indent, record_ctx)
                }
            }

            Element::ContentList { source, template } => {
                let mut output = String::new();
                if let Some(list) = self.content_defs.get_list(source) {
                    for item in list {
                        if let ContentValue::Record(record) = item {
                            output.push_str(&self.render_element(template, indent, Some(record)));
                        }
                    }
                }
                output
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
        record_ctx: Option<&HashMap<String, String>>,
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
            let prop_str = self.render_prop(key, value, record_ctx);
            output.push_str(&format!(" {}", prop_str));
        }

        // Check for text prop (used as inner text)
        let text_content = props.get("text").map(|v| self.prop_value_to_string(v, record_ctx));

        let has_children = !children.is_empty() || text_content.is_some();

        if has_children {
            output.push_str(">\n");

            // Render text content if present
            if let Some(text) = text_content {
                output.push_str(&format!("{}{}\n", " ".repeat(indent + 2), text));
            }

            // Render children
            for child in children {
                output.push_str(&self.render_element(child, indent + 2, record_ctx));
            }

            // Closing tag
            output.push_str(&format!("{}</{}>\n", indent_str, tag));
        } else {
            // Self-closing tag
            output.push_str(" />\n");
        }

        output
    }

    fn render_prop(&self, key: &str, value: &PropValue, record_ctx: Option<&HashMap<String, String>>) -> String {
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
                            // Check if it's an external URL
                            if let Some(path) = &asset.path {
                                if path.starts_with("http://") || path.starts_with("https://") {
                                    // External URL - use directly as string
                                    format!("{}=\"{}\"", key, path)
                                } else {
                                    // Local asset - use imported variable reference
                                    format!("{}={{{}}}", key, asset_name)
                                }
                            } else {
                                format!("{}={{{}}}", key, asset_name)
                            }
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
            PropValue::Content(content_name) => {
                // Look up content and inline it as a string
                if let Some(text) = self.content_defs.get_str(content_name) {
                    format!("{}=\"{}\"", key, text)
                } else {
                    format!("{}=\"\"", key)
                }
            }
            PropValue::ContentField(field_name) => {
                // Look up field in current record context
                if let Some(record) = record_ctx {
                    if let Some(value) = record.get(field_name) {
                        format!("{}=\"{}\"", key, value)
                    } else {
                        format!("{}=\"\"", key)
                    }
                } else {
                    format!("{}=\"\"", key)
                }
            }
        }
    }

    fn prop_value_to_string(&self, value: &PropValue, record_ctx: Option<&HashMap<String, String>>) -> String {
        match value {
            PropValue::Str(s) => s.clone(),
            PropValue::Num(n) => n.to_string(),
            PropValue::Bool(b) => b.to_string(),
            PropValue::Var(var_name) => format!("{{{}}}", var_name),
            PropValue::Asset(asset_name) => {
                if let Some(asset) = self.asset_defs.get(asset_name) {
                    match asset.kind {
                        AssetKind::Image => {
                            // Check if external URL
                            if let Some(path) = &asset.path {
                                if path.starts_with("http://") || path.starts_with("https://") {
                                    path.clone()
                                } else {
                                    format!("{{{}}}", asset_name)
                                }
                            } else {
                                format!("{{{}}}", asset_name)
                            }
                        }
                        _ => asset.url.clone().unwrap_or_default(),
                    }
                } else {
                    format!("{{{}}}", asset_name)
                }
            }
            PropValue::Content(content_name) => {
                self.content_defs.get_str(content_name).cloned().unwrap_or_default()
            }
            PropValue::ContentField(field_name) => {
                if let Some(record) = record_ctx {
                    record.get(field_name).cloned().unwrap_or_default()
                } else {
                    String::new()
                }
            }
        }
    }
}
