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

### Element (recursive tree)
```rust
enum Element {
    Text(String),
    Node { tag, class_name, props, children: Vec<Box<Element>> },
    ComponentRef { component, props, children },
}
```

### PropValue
```rust
enum PropValue {
    Str(String),
    Num(f64),
    Bool(bool),
    Var(String),      // Variable reference
    Asset(String),    // Asset lookup
}
```

### ComponentDef (presets)
```rust
struct ComponentDef {
    name: String,
    tag: String,
    class_name: Option<String>,
    default_props: HashMap<String, PropValue>,
}
```

### AssetDef (asset lookup)
```rust
struct AssetDef {
    name: String,
    kind: AssetKind,  // Image, Youtube, Video, Audio
    path: Option<String>,
    url: Option<String>,
}
```

## Usage

```rust
use degenproto_engine::{ProtoIndex, RouterJsx, ViewProto, ViewJsx, ComponentDefs, AssetDefs};

// Router generation
let index = ProtoIndex::from_file("proto/index.ron")?;
let router = RouterJsx::from_proto_index(index);
let jsx = router.to_string();

// View generation
let view = ViewProto::from_file("proto/home.ron")?;
let components = ComponentDefs::from_file("proto/component_defs.ron")?;
let assets = AssetDefs::from_file("proto/assets_def.ron")?;
let view_jsx = ViewJsx::new(view, components, assets);
let jsx = view_jsx.to_string();
```

## Consumer

Used by `yeriko-dj-web` (sibling directory) as a path dependency. The `degenbuild` binary in that repo calls this library.

## Design Decisions

- **Pure RON** - Uses `Box<Element>` for recursive nesting instead of JSON
- **Lookup tables** - ComponentDefs and AssetDefs allow referencing by name
- **Auto-imports** - Image assets automatically generate import statements
- **URL expansion** - Youtube/Video assets expand to their URLs inline
