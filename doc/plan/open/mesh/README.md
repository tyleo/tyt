# Mesh plan

This plan rewrites [`vxl mesh`](mesh.md) which makes geometry and palette-atlas
textures. The plan keeps the geometry core and redoes everything around it: the
shipped map surface retires wholesale and will be deleted from the codebase. The
new surface is a small expression language. A material map is a value that you
write. A value profile names a reusable set of values. An output profile
describes a full run in `.vxlconfig`. The plan also adds parts `vxl mesh`
never had.

1. [`vxl mesh`](mesh.md): the command reference, from its arguments to the
   glTF a run emits.
2. [Value language](value-language.md): the expression language for material
   values. Expressions read and define values, and writers put the results in
   images, JSON, and the mesh's own material.
3. [Profile language](profile-language.md): the value and output profiles,
   defined in a new `.vxlconfig` file or built into the binary.
4. [Worked examples](examples.md): Examples showing a profile, its command line,
   and the glTF that it makes.
5. [Implementation](implementation.md): how the tool changes land, from the new
   language crate to the code the rewrite deletes.
