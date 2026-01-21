# degenproto_engine

Rust library for parsing RON configuration files and generating React/JSX code.

## Purpose

This is the engine that powers the "boil down to RON" approach - defining a website declaratively in RON files, then generating all the React code from those definitions.

## Structure

- `src/lib.rs` - Core types: `ProtoIndex`, `Layout`, `Route`, and RON parsing
- `src/view_proto.rs` - View types: `Element`, `PropValue`, `ComponentDef`, `AssetDef`
- `src/router_jsx.rs` - `RouterJsx` struct that generates `router/index.jsx`
- `src/view_jsx.rs` - `ViewJsx` struct that generates view components

## Core Types

### Element (recursive tree with Box)
```rust
enum Element {
    Text(String),
    Node { tag, class_name, props, children: Vec<Box<Element>> },
    ComponentRef { component, props, children },  // References component_defs by name
}
```

### PropValue
```rust
enum PropValue {
    Str(String),      // Regular string: "hello"
    Num(f64),         // Number: 560
    Bool(bool),       // Boolean: true/false
    Var(String),      // JS variable reference: {someVar}
    Asset(String),    // Asset lookup - resolves from AssetDefs
}
```

### ComponentDef (presets in component_defs.ron)
```rust
struct ComponentDef {
    name: String,
    tag: String,
    class_name: Option<String>,
    default_props: HashMap<String, PropValue>,
}
```

### AssetDef (assets in assets_def.ron)
```rust
enum AssetKind { Image, Youtube, Video, Audio }

struct AssetDef {
    name: String,
    kind: AssetKind,
    path: Option<String>,  // For Image - generates import
    url: Option<String>,   // For Youtube/Video - inlines directly
}
```

## Asset Resolution

When `ViewJsx` encounters `Asset("SomeName")`:

1. **Image assets**: Auto-generates an import statement and renders as `{SomeName}`
   ```jsx
   import SomeName from '@/assets/path.jpg';
   // ...
   <img src={SomeName} />
   ```

2. **Youtube/Video assets**: Inlines the URL directly (no import)
   ```jsx
   <iframe src="https://youtube.com/embed/..." />
   ```

## Usage

```rust
use degenproto_engine::{ProtoIndex, RouterJsx, ViewProto, ViewJsx, ComponentDefs, AssetDefs};

// Load shared definitions once
let components = ComponentDefs::from_file("proto/component_defs.ron")?;
let assets = AssetDefs::from_file("proto/assets_def.ron")?;

// Router generation
let index = ProtoIndex::from_file("proto/index.ron")?;
let router = RouterJsx::from_proto_index(index);
fs::write("src/router/index.jsx", router.to_string())?;

// View generation (for each route)
let view = ViewProto::from_file("proto/home.ron")?;
let view_jsx = ViewJsx::new(view, components.clone(), assets.clone());
fs::write("src/views/welcome/Home.jsx", view_jsx.to_string())?;
```

## Consumer

Used by `yeriko-dj-web` (sibling directory) as a path dependency. The `degenbuild` binary iterates over routes in index.ron and generates views for any that have matching .ron files.

## Design Decisions

- **Pure RON** - Uses `Box<Element>` for recursive nesting instead of JSON
- **Lookup tables** - ComponentDefs and AssetDefs allow referencing by name
- **Auto-imports** - Image assets automatically generate import statements
- **URL expansion** - Youtube/Video assets expand to their URLs inline
- **Data-driven** - degenbuild reads index.ron, nothing hardcoded
