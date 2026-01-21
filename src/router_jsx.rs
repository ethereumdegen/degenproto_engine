use crate::{Layout, ProtoIndex, Route};
use std::collections::{HashMap, HashSet};

pub struct RouterJsx {
    pub layouts: Vec<Layout>,
    pub routes: Vec<Route>,
}

struct ImportMap {
    // path -> component_name
    path_to_name: HashMap<String, String>,
    used_names: HashSet<String>,
}

impl ImportMap {
    fn new() -> Self {
        Self {
            path_to_name: HashMap::new(),
            used_names: HashSet::new(),
        }
    }

    fn add_layout(&mut self, layout: &Layout) -> String {
        if let Some(name) = self.path_to_name.get(&layout.path) {
            return name.clone();
        }

        let base_name = format!("{}Layout", capitalize(&layout.name));
        let name = self.unique_name(&base_name);
        self.path_to_name.insert(layout.path.clone(), name.clone());
        self.used_names.insert(name.clone());
        name
    }

    fn add_route(&mut self, route: &Route) -> String {
        if let Some(name) = self.path_to_name.get(&route.path) {
            return name.clone();
        }

        let base_name = capitalize(&route.name);
        let name = self.unique_name(&base_name);
        self.path_to_name.insert(route.path.clone(), name.clone());
        self.used_names.insert(name.clone());
        name
    }

    fn unique_name(&self, base: &str) -> String {
        if !self.used_names.contains(base) {
            return base.to_string();
        }

        let mut counter = 2;
        loop {
            let candidate = format!("{}{}", base, counter);
            if !self.used_names.contains(&candidate) {
                return candidate;
            }
            counter += 1;
        }
    }

    fn get(&self, path: &str) -> Option<&String> {
        self.path_to_name.get(path)
    }
}

impl RouterJsx {
    pub fn from_proto_index(index: ProtoIndex) -> Self {
        Self {
            layouts: index.layouts,
            routes: index.routes,
        }
    }

    pub fn to_string(&self) -> String {
        let mut imports = String::new();
        let mut route_elements = String::new();
        let mut import_map = ImportMap::new();

        // Register all layouts and routes first to get unique names
        for layout in &self.layouts {
            import_map.add_layout(layout);
        }
        for route in &self.routes {
            import_map.add_route(route);
        }

        // Import useRoutes
        imports.push_str("import { useRoutes } from \"react-router-dom\";\n");

        // Import layouts
        for layout in &self.layouts {
            let component_name = import_map.get(&layout.path).unwrap();
            imports.push_str(&format!(
                "import {} from \"../{}\";\n",
                component_name,
                layout.path
            ));
        }

        imports.push_str("\n");

        // Import views (deduplicated by path)
        let mut seen_paths: HashSet<String> = HashSet::new();
        for route in &self.routes {
            if seen_paths.contains(&route.path) {
                continue;
            }
            seen_paths.insert(route.path.clone());

            let component_name = import_map.get(&route.path).unwrap();
            imports.push_str(&format!(
                "import {} from \"../{}\";\n",
                component_name,
                route.path
            ));
        }

        // Build route configuration
        // Group routes by layout
        let mut layout_routes: HashMap<String, Vec<&Route>> = HashMap::new();
        let mut no_layout_routes: Vec<&Route> = Vec::new();

        for route in &self.routes {
            if let Some(layout) = &route.layout {
                layout_routes.entry(layout.clone()).or_default().push(route);
            } else {
                no_layout_routes.push(route);
            }
        }

        route_elements.push_str("  const routes = [\n");

        // Routes with layouts
        for layout in &self.layouts {
            if let Some(routes) = layout_routes.get(&layout.name) {
                let layout_component = import_map.get(&layout.path).unwrap();
                route_elements.push_str(&format!(
                    "    {{\n      path: \"/\",\n      element: <{} />,\n      children: [\n",
                    layout_component
                ));

                for route in routes {
                    let component_name = import_map.get(&route.path).unwrap();
                    route_elements.push_str(&format!(
                        "        {{\n          path: \"{}\",\n          element: <{} />,\n        }},\n",
                        route.url,
                        component_name
                    ));
                }

                route_elements.push_str("      ],\n    },\n");
            }
        }

        // Routes without layouts
        for route in no_layout_routes {
            let component_name = import_map.get(&route.path).unwrap();
            route_elements.push_str(&format!(
                "    {{\n      path: \"{}\",\n      element: <{} />,\n    }},\n",
                route.url,
                component_name
            ));
        }

        route_elements.push_str("  ];\n");

        format!(
            r#"{}
function Router() {{
{}
  return useRoutes(routes);
}}

export default Router;
"#,
            imports,
            route_elements
        )
    }
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
    }
}
