#version 450
#extension GL_EXT_shader_16bit_storage : require
#extension GL_EXT_shader_8bit_storage : require

struct Node {
    uint8_t _padding1;
    uint8_t freemask;
    uint16_t _padding2;
    uint children;
    uint16_t data[8];
};



layout(location=0) in vec3 Vertex_Position;

layout(set = 0, binding = 0) uniform Camera {
    mat4 ViewProj;
};

layout(set = 3, binding = 0) readonly buffer Chunk {
    vec4 bounding_box;
    Node nodes[];
};

void main() {
    vec3 transformed_position = Vertex_Position * bounding_box.w + bounding_box.xyz;;
    gl_Position = ViewProj * vec4(transformed_position, 1.0);
}
