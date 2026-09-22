# meshdoc

The in-memory mesh model at the bottom of the mesh stack. Each format has a
`-meshdoc` bridge crate that reads its files into a `MeshMain` and writes one
back out.

## Architecture

`MeshMain<T>` holds a document the way every format sees it. Files are the
loose files that land beside the document. Images hold encoded bytes, their
own or a file's. Textures sample images. Materials bind the
metallic-roughness factors and the textures behind them. Objects are
ordered lists of primitives, each a triangle list over its own vertex
streams drawing one material. Materials and objects carry named properties
for what the model does not place. Hierarchy nodes place objects and each
other under transforms. `T` is the ext, the format-specific state a bridge
carries alongside the document.

Each kind of entity lives in an id pool. Ids are branded, so a texture id
cannot stand in for an image id. Ids are meaningful only within the state
that issued them. Because every mutation checks the cross-references it
could break, a state reached through the public API never violates a
referential rule. `validate` checks a whole state at once.

The model is shaped by what the popular mesh formats carry, glTF 2.0 first.
Positions are in meters on glTF's frame: Y-up, right-handed, with +Z toward
the viewer. Triangles wind counter-clockwise seen from outside. Node transforms stay on the nodes; nothing bakes into positions.

The sections below build one document up from an image and read it back.

## Files

A `MeshFile` is a relative name and bytes. A writer lands every file beside
the document's primary file under its name, so a JSON sidecar or a texture
that must stay a file on disk has a home in the document. Names are unique.
An image can read its bytes from a file, and a property can point at one.

```rust
let mut main: MeshMain = MeshMain::default();

let file_id = main.retain_file(MeshFile {
    name: "values.json".to_owned(),
    bytes: json_bytes,
})?;
```

## Images and Textures

A `MeshImage` is a media type, PNG or JPEG, over a `MeshImageSource`: its
own encoded bytes, which a container embeds, or one of the document's files,
which a writer references by name. The document never decodes pixels.
`image_bytes` reads either source. A `MeshTexture` samples an image with
its filters and wrap modes.

```rust
let image_id = main.retain_image(MeshImage {
    name: "atlas".to_owned(),
    media_type: MeshImageMediaType::Png,
    source: MeshImageSource::Bytes(png_bytes),
})?;
let texture_id = main.retain_texture(MeshTexture::new(image_id))?;
```

## Materials

A `MeshMaterial` carries the metallic-roughness set: the factors and their
textures for base color, metallic roughness, normal, occlusion, and
emissive, the alpha mode and cutoff, double sidedness, and the common
extensions, emissive strength, ior, and transmission. A texture reference
points to a texture and the UV stream the drawing primitives sample it
through. The `material` module fixes the shared names, such as `BASE_COLOR`
for `baseColor`, their defaults, and their ranges.

```rust
let material_id = main.retain_material(MeshMaterial {
    base_color_texture: Some(MeshTextureRef {
        texture_id,
        uv_stream_id: U32Id::from_u32(0),
    }),
    ..MeshMaterial::new("skin".to_owned())
})?;
```

## Properties

A material and an object each carry named `MeshProperty` entries for what
the model does not place. A `MeshPropertyValue` is a bool, an integer, a
float, a string, a list of any of those, a list of float rows, a texture
reference, or one of the document's files. Names are unique within their
owner, floats are finite, and every texture and file referenced is live. A
bridge writes them where its format keeps application data and reads them
back.

```rust
let material_id = main.retain_material(MeshMaterial {
    properties: vec![MeshProperty {
        name: "rows".to_owned(),
        value: MeshPropertyValue::File(file_id),
    }],
    ..MeshMaterial::default()
})?;

main.set_object_properties(object_id, vec![MeshProperty {
    name: "albedo".to_owned(),
    value: MeshPropertyValue::FloatRows(rows),
}])?;
```

## Objects

A `MeshObject` is a name, properties, and an ordered list of primitives. A
`MeshPrimitive` holds positions and triangles, with a name, optional
normals, tangents, UV streams, vertex colors, and further named attributes.
Every stream holds one entry per vertex. A further attribute's
`MeshAttributeComponents` are floats, 8-bit, or 16-bit unsigned integers.
Triangle corners reference vertices by id. The primitive draws one material
and must carry every UV stream that material's textures sample.

```rust
let mut primitive = MeshPrimitive::new(positions, triangles)?;
primitive.push_uv_stream(uvs)?;
primitive.push_vertex_attribute(MeshVertexAttribute {
    name: "_PALETTE".to_owned(),
    width: 1,
    components: MeshAttributeComponents::U8(swatch_indices),
})?;
primitive.set_material_id(Some(material_id));

let mut object = MeshObject::new("body".to_owned());
object.retain_primitive(primitive);
let object_id = main.retain_object(object)?;
```

## Hierarchy

A `MeshHierarchyNode` carries a transform and places child nodes and
objects. Nodes form a DAG: one node can sit under several parents. The
document's roots are the nodes nothing places.

```rust
let root_id = main.retain_hierarchy_node(MeshHierarchyNode {
    name: "root".to_owned(),
    child_object_ids: vec![object_id],
    ..Default::default()
})?;
main.push_root_hierarchy_node_id(root_id)?;

main.validate()?;
```

## Reading Back

The read API lives on `MeshState`, which `MeshMain` forwards. Look an entity
up by id or walk a listing in order.

```rust
let object = main.object(object_id).expect("retained above");
let (_, primitive) = object.iter_primitives().next().expect("one primitive");
assert_eq!(primitive.material_id(), Some(material_id));
assert_eq!(primitive.vertex_count(), 3);
```

## Releasing and Compaction

A release refuses while anything still references the entity. A release that
goes through leaves a hole in its id pool. `gc` compacts every pool back to
contiguous ids. Call it once before a save to keep saves deterministic. It
returns a `MeshGcRemap` that translates any ids held outside the main.

```rust
let spare_id = main.retain_material(MeshMaterial::default())?;
main.release_material(spare_id)?;

main.gc()?;
```

## The Ext

Every format keeps state that meshdoc has no home for: glTF's scenes,
cameras, skins, animations, extras, and unknown extensions. Dropping that
state would make a round trip through meshdoc lossy. A bridge's loader stores
its format's ext as the state's `T`, and its writer reads the ext back. A
document loaded from a format writes back to that format exactly. A bare
state writes from the document. The core never reads the ext.

A format ext keeps an entry per entity, keyed by the entity's id. A mutation
that retains or releases an entity would leave it stale. The main tells the
ext through `MeshExt`, and every mutation carries that bound:

1. a retain fires its hook after the mutation
2. a release fires its hook before the mutation, once every check has passed
3. `gc` fires `did_gc` with the id remap

Each hook sees the `MeshState` and can write a complete entry for the entity
on the spot. A hook returns a `Result`. A `will` hook's error refuses the
mutation and leaves the main unchanged. Every hook defaults to `Ok(())`.
`()` ignores them all. The hooks are the whole trait. How an ext persists,
and how a boxed one downcasts or clones, belongs to the crate that converts
between formats.

```rust
#[derive(Clone, Debug, Default)]
struct Released(Vec<String>);

impl MeshExt for Released {
    fn object_will_release(
        &mut self,
        state: &MeshState,
        object_id: U32Id<BMeshObject>,
    ) -> Result<()> {
        let object = state
            .object(object_id)
            .expect("a released object is live");
        self.0.push(object.name().to_owned());
        Ok(())
    }
}

let mut main: MeshMain<Released> = MeshMain::default();
let first_id = main.retain_object(first)?;
main.retain_object(second)?;
main.release_object(first_id)?;
assert_eq!(main.ext().0, ["first"]);
```

`take_ext` takes the ext off a main. The result is a `TakenExt` holding the
bare `MeshMain<()>` beside the ext. `put_ext` puts an ext on a bare main.
Together they change a state's ext type and move the document over
unchanged.

```rust
let TakenExt { main: bare, ext: released } = main.take_ext();
let main: MeshMain<Released> = bare.put_ext(released);
```

## Checks

The `check` module holds the check vocabulary: a `MeshCheck` carries a
check's name and its `MeshCheckStatus`. A format's crate runs its checks
over its encoding and reports them in this form.
