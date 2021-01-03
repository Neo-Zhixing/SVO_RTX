#version 460
#extension GL_EXT_shader_16bit_storage : require
#extension GL_EXT_shader_8bit_storage : require

layout(location=0) out vec4 f_color;
layout(location=0) in vec2 v_tex_coords;

struct PerspectiveProjection {
    float fov;
    float aspect_ratio;
    float near;
    float far;
};

struct OctreeNode {
    uint8_t _padding1;
    uint8_t freemask;
    uint16_t _padding2;
    uint children;
    uint16_t data[8];
};

layout(set = 0, binding = 0) uniform Camera {
    mat4 ViewProj;
};
layout(set = 1, binding = 0) uniform Camera3dProjection {
    mat4 transform;
    PerspectiveProjection projection;
};
layout(set = 2, binding = 0) readonly buffer Chunk {
    OctreeNode nodes[];
};


void main() {
    vec2 pixel = vec2(v_tex_coords.x * projection.aspect_ratio, v_tex_coords.y) * tan(projection.fov / 2);
    vec4 pixel_camera_space = vec4(pixel, -projection.near, 1.0);
    vec4 pixel_world_space_homo = transform * pixel_camera_space;
    vec3 pixel_world_space = pixel_world_space_homo.xyz / pixel_world_space_homo.w;
    vec4 origin_world_space_homo = transform * vec4(0.0, 0.0, 0.0, 1.0);

    vec3 origin_world_space = origin_world_space_homo.xyz / origin_world_space_homo.w;
    vec3 ray_world_space = normalize(pixel_world_space - origin_world_space);

    f_color = vec4(ray_world_space.x, 0.0, 0.0, 1.0);
}
