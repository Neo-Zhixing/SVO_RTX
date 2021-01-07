#version 460
#extension GL_EXT_shader_16bit_storage : require
#extension GL_EXT_shader_8bit_storage : require

layout(location=0) out vec4 f_color;

struct PerspectiveProjection {
    float fov;
    float aspect_ratio;
    float near;
    float far;
    vec2 dimensions;
};

struct Node {
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
    Node nodes[];
};

struct Ray {
    vec3 origin;
    vec3 dir;
};

Ray generate_ray() {
    vec2 pixel_ndc = gl_FragCoord.xy / projection.dimensions;
    vec2 pixel_screen = pixel_ndc * 2 - 1;
    vec2 pixel = pixel_screen * tan(projection.fov / 2);
    pixel.x *= projection.aspect_ratio;
    pixel.y *= -1;

    vec4 pixel_camera_space = vec4(pixel, -1, 1.0);
    vec4 pixel_world_space_homo = transform * pixel_camera_space;
    vec3 pixel_world_space = pixel_world_space_homo.xyz / pixel_world_space_homo.w;

    Ray ray;
    ray.origin = transform[3].xyz;
    ray.dir = normalize(pixel_world_space - ray.origin);
    return ray;
}
vec2 intersectAABB(vec3 origin, vec3 dir, vec4 box) {
    vec3 box_min = box.xyz;
    vec3 box_max = box_min + box.w;
    vec3 tMin = (box_min - origin) / dir;
    vec3 tMax = (box_max - origin) / dir;
    vec3 t1 = min(tMin, tMax);
    vec3 t2 = max(tMin, tMax);
    float t_min = max(max(t1.x, t1.y), t1.z);
    float t_max = min(min(t2.x, t2.y), t2.z);
    return vec2(t_min, t_max);
}

bool containsAABB(vec3 point, vec4 box) {
    vec3 min = box.xyz;
    vec3 max = min + box.w;

    vec3 s = step(min, point) - step(max, point);
    bvec3 bs = bvec3(s);
    return all(bs);
}

// Given a mask an a location, returns n where the given '1' on the location
// is the nth '1' counting from the Least Significant Bit.
uint mask_location_nth_one(uint mask, uint location) {
    return bitCount(mask & ((1 << location) - 1));
}


uint material_at_position(inout vec4 box, vec3 position) {
    uint node_index = 0; // Assume root node

    while(true) {
        // start
        // Calculate new box location
        box.w = box.w / 2;
        vec3 box_midpoint = box.xyz + box.w;
        vec3 s = step(box_midpoint, position);
        box.xyz = box.xyz + s * box.w;


        uint child_index = uint(dot(s, vec3(4,2,1)));
        uint freemask = uint(nodes[node_index].freemask);
        if ((freemask & (1 << child_index)) == 0) {
            // is a leaf node
            return uint(nodes[node_index].data[child_index]);
        } else {
            // has children
            uint child_offset = mask_location_nth_one(freemask, child_index);
            node_index = nodes[node_index].children + child_offset;
        }
    }
}


void main() {
    Ray ray = generate_ray();

    vec4 box = vec4(0,0,0,1);
    vec2 intersection = intersectAABB(ray.origin, ray.dir, box);
    vec3 entry_point = ray.origin + intersection.x * ray.dir + sign(ray.dir) * box.w * 0.00001;
    uint material_id = 0;

    for(uint counter = 0; counter < 50 && containsAABB(entry_point, box); counter++) {
        vec4 hitbox = box;
        material_id = material_at_position(hitbox, entry_point);
        if (material_id > 0) {
            // get the depth info from entry_point
            vec4 entry_point_camera_space = ViewProj * vec4(entry_point, 1.0);
            gl_FragDepth = entry_point_camera_space.z/entry_point_camera_space.w;
            break;
        }
        // calculate the next t_min
        vec2 new_intersection = intersectAABB(entry_point, ray.dir, hitbox);

        entry_point += ray.dir * new_intersection.y + sign(ray.dir) * hitbox.w * 0.00001;
    }

    if (material_id == 0) {
        f_color = vec4(0.0, 0.0, 0.0, 1.0);
    } else if (material_id == 2) {
        f_color = vec4(1.0, 0.0, 0.0, 1.0);
    } else {
        f_color = vec4(0.0, 1.0, 0.0, 1.0);
    }
}
