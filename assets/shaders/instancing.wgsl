/*
struct OurStruct {
    color: vec4f,
    offset: vec4f,
};

struct OtherStruct {
    scale: vec2f,
};
*/
#import bevy_render::view::View

@group(0) @binding(0) var<uniform> view: View;

struct Vertex {
    @location(0) position: vec2f,
    @location(1) color: vec4f,
    @location(2) offset: vec2f,
    @location(3) scale: vec2f,
};

struct VertexOutput {
    @builtin(position) position: vec4f,
    @location(0) color: vec4f,
};

// @group(0) @binding(0) var<storage, read> our_struct: array<OurStruct>;
// @group(0) @binding(1) var<storage, read> other_struct: array<OtherStruct>;

@vertex
fn vs(
    vertex: Vertex,
    //@builtin(instance_index) instance_index: u32
) -> VertexOutput {
    /*
    let other_struct = other_structs[instance_index];
    let our_struct = our_structs[instance_index];
    */

    var vertex_output: VertexOutput;
    /*
    vertex_output.position = vec4f(
        vertex.position * other_struct.scale * our_struct.offset, 0.0, 1.0
    );
    vertex_output.color = our_struct.color;
    */
    vertex_output.position = view.clip_from_world * vec4f(
        vertex.position * vertex.scale + vertex.offset, 0.0, 1.0
    );
    vertex_output.color = vertex.color;
    return vertex_output;
}

@fragment
fn fs(vertex_output: VertexOutput) -> @location(0) vec4f {
    return vertex_output.color;
}