# Mesh plan

The plan for finishing [`vxl mesh`](mesh.md). The command ships today
with geometry and palette-atlas textures behind preset flags. This plan
completes the API. The material surface becomes a small expression
language, so a material map is a value you write, with value profiles
naming reusable sets of values and an output profile spelling a whole
run in `.vxlconfig`. The rest of the command lands beside them: the
unwrap atlas, computed occlusion, primitives and materials, vertex
attributes, and the mesh palettes.
The pages are the
design; the schedule is an
[open question](implementation.md#open-questions).

1. [`vxl mesh`](mesh.md): the command, every argument and option,
   primitives and materials, the two atlases, the UV streams, and
   the palettes.
2. [Value language](value-language.md): values, shapes, domains,
   booleans, writers, slots, and the grammar.
3. [Profile language](profile-language.md): the value and output
   profiles, loading, and the built-ins.
4. [Implementation](implementation.md): the language crate, the
   ty-preferences work, the retired flags, the code deletions, and the
   open questions.
5. [Open](open.md): the open calls, what settles them, and the doc
   TODOs.
