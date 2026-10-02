# Render reference

_The reference pages of the
[voxel rendering plan](../../plan/closed/voxel-rendering/README.md)._

`vxl object render` draws the selected objects of a voxel document into one
image per view, inline in the terminal or as PNGs beside the input. The
pages below hold what a user of the command reads. The plan keeps the design
and the decisions.

1. [`vxl object render`](render.md): the command reference, from its arguments
   to the files a run writes.
2. [The contract](contract.md): what the image of a scene is, from the frame
   and the surface to the output encoding. Every renderer implements it.
3. [Profile language](profile-language.md): the profiles that hold views and
   lights, defined in `.vxlconfig` or built into the binary.
