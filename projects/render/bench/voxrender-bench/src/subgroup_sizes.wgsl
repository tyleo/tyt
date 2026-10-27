struct Image {
    width: u32,
    height: u32,
}

@group(0) @binding(0) var<uniform> image: Image;
@group(0) @binding(1) var<storage, read_write> sizes: array<atomic<u32>, 2>;

@compute @workgroup_size(8, 8)
fn main(
    @builtin(global_invocation_id) id: vec3<u32>,
    @builtin(subgroup_size) size: u32,
) {
    if id.x >= image.width || id.y >= image.height {
        return;
    }

    atomicMin(&sizes[0], size);
    atomicMax(&sizes[1], size);
}
