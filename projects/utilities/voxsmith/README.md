# voxsmith

Operations over `voxcore` states. `operations` holds one module and feature per `vxl` command: the info, validate, hierarchy, and palette reports, glTF meshing, voxelizing, and the object pruning behind `to`. `utilities` holds palette reduction, dithering, color spaces, and the object selectors. `dependencies` holds the traits the operations take from the caller: the mesh writer's PNG and base64 encoders. `DependenciesImpl`, behind the `impl` feature, binds those encoders over `png` and `base64`. The file formats live in `voxconv`.
