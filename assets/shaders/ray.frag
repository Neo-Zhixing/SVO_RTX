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

struct Ray {
    vec3 origin;
    vec3 dir;
};

struct AABB {
    vec3 min;
    vec3 max;
};

Ray generate_ray() {
    vec2 pixel = vec2(v_tex_coords.x * projection.aspect_ratio, v_tex_coords.y) * tan(projection.fov / 2);
    vec4 pixel_camera_space = vec4(pixel, -projection.near, 1.0);
    vec4 pixel_world_space_homo = transform * pixel_camera_space;
    vec3 pixel_world_space = pixel_world_space_homo.xyz / pixel_world_space_homo.w;
    vec4 origin_world_space_homo = transform * vec4(0.0, 0.0, 0.0, 1.0);

    Ray ray;
    ray.origin = origin_world_space_homo.xyz / origin_world_space_homo.w;
    ray.dir = normalize(pixel_world_space - ray.origin);
    return ray;
}
void intersectAABB(Ray ray, AABB box, out float t_min, out float t_max) {
    vec3 tMin = (box.min - ray.origin) / ray.dir;
    vec3 tMax = (box.max - ray.origin) / ray.dir;
    vec3 t1 = min(tMin, tMax);
    vec3 t2 = max(tMin, tMax);
    t_min = max(max(t1.x, t1.y), t1.z);
    t_max = min(min(t2.x, t2.y), t2.z);
}

void main() {
    Ray ray = generate_ray();

    AABB aabb;
    aabb.min = vec3(0,0,0);
    aabb.max = vec3(1,1,1);

    float t_min, t_max;
    intersectAABB(ray, aabb, t_min, t_max);
    vec3 entry_point = ray.origin + ray.dir * t_min;

    if (0 < t_min && t_min < t_max) {
        // intersected
        f_color = vec4(1.0, 0.0, 0.0, 1.0);
    } else {
        // no intersection
        f_color = vec4(0.0, 0.0, 0.0, 1.0);
    }

}
