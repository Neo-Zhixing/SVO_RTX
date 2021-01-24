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
    vec2 _padding;
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

struct PointLight {
    vec4 Color;
    vec4 pos;
};
layout(set = 1, binding = 0) uniform Camera3dProjection {
    mat4 transform;
    PerspectiveProjection projection;
};
layout(set = 1, binding = 1) uniform texture2DArray TextureRepo;
layout(set = 1, binding = 2) uniform sampler TextureRepoSampler;
layout(set = 1, binding = 3) uniform Lights {
    vec4 AmbientLightColor;
    vec4 SunLightColor;
    vec3 SunLightDir;
    uint PointLightCount;
    PointLight lights[];
};
layout (constant_id = 0) const uint MAX_ITERATION_VALUE = 100;

layout(set = 2, binding = 0) readonly buffer Chunk {
    vec4 bounding_box;
    Node nodes[];
};

struct Material {
    float scale;
    uint16_t diffuse;
    uint16_t normal;
    float _reserved1;
    float _reserved2;
};
struct ColoredMaterial {
    float scale;
    uint16_t diffuse;
    uint16_t normal;
    float _reserved1;
    float _reserved2;
    vec4 palette[256];
};
layout(set = 2, binding = 1) readonly buffer ColoredMaterials {
    ColoredMaterial coloredMaterials[];
};
layout(set = 2, binding = 2) readonly buffer Materials {
    Material regularMaterials[];
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

vec3 cubed_normalize(vec3 dir) {
    vec3 dir_abs = abs(dir);
    float max_element = max(dir_abs.x, max(dir_abs.y, dir_abs.z));
    return -sign(dir) * step(max_element, dir_abs);
}


uint RayMarch(vec4 initial_box, Ray ray, out vec3 hitpoint, out vec4 hitbox, out uint iteration_times) {
    hitbox = initial_box;
    vec2 intersection = intersectAABB(ray.origin, ray.dir, hitbox);
    vec3 entry_point = ray.origin + max(0, intersection.x) * ray.dir;
    vec3 test_point = entry_point + ray.dir * hitbox.w * 0.000001;
    uint material_id = 0;

    uint counter;
    for(counter = 0;; counter++) {
        vec4 entry_point_camera_space = ViewProj * vec4(entry_point, 1.0);
        gl_FragDepth = ((entry_point_camera_space.z/entry_point_camera_space.w) + 1.0) * 0.5;
        if (counter >= MAX_ITERATION_VALUE) {
            // Ray tracing failed
            break;
        }
        if (!containsAABB(test_point, bounding_box)) {
            // Outside the box
            discard;
        }
        hitbox = initial_box;
        material_id = material_at_position(hitbox, test_point);
        if (material_id > 0) {
            // Hit some materials
            break;
        }
        // calculate the next t_min
        vec2 new_intersection = intersectAABB(entry_point, ray.dir, hitbox);

        entry_point = entry_point + ray.dir * new_intersection.y;
        test_point = entry_point + sign(ray.dir) * hitbox.w * 0.0001;
    }
    hitpoint = entry_point;
    iteration_times = counter;
    return material_id;
}

// Return true if occluded
bool RayMarchTest(vec4 initial_box, Ray ray) {
    vec4 hitbox = initial_box;
    vec2 intersection = intersectAABB(ray.origin, ray.dir, hitbox);
    vec3 entry_point = ray.origin + max(0, intersection.x) * ray.dir;
    vec3 test_point = entry_point + ray.dir * hitbox.w * 0.000001;
    uint material_id = 0;

    for(uint counter = 0; counter < 50 && containsAABB(test_point, bounding_box); counter++) {
        hitbox = initial_box;
        material_id = material_at_position(hitbox, test_point);
        if (material_id > 0) {
            // get the depth info from entry_point
            return true;
        }
        // calculate the next t_min
        vec2 new_intersection = intersectAABB(test_point, ray.dir, hitbox);

        entry_point = test_point + ray.dir * new_intersection.y;
        test_point = entry_point + sign(ray.dir) * hitbox.w * 0.0001;
    }
    return false;
}
void main() {
    Ray ray = generate_ray();

    float depth;
    vec3 hitpoint;
    vec4 hitbox;
    uint iteration_times;
    uint voxel_id = RayMarch(bounding_box, ray, hitpoint, hitbox, iteration_times);
    float iteration = float(iteration_times) / float(MAX_ITERATION_VALUE); // 0 to 1

    #ifndef MYMATERIAL_ALWAYS_BLUE
    f_color = vec4(iteration, iteration, iteration, 1.0);
    #else

    vec3 normal = cubed_normalize(hitpoint - (hitbox.xyz + hitbox.w/2));
    vec2 texcoords = vec2(
        dot(vec3(hitpoint.z, hitpoint.x, -hitpoint.x), normal),
        dot(-sign(normal) * vec3(hitpoint.y, hitpoint.z, hitpoint.y), normal)
    );
    vec4 output_color;
    uint diffuse_texture_id;
    float scale;

    // Calculate ambient
    vec3 light_color = AmbientLightColor.rgb;

    // Test Sunlight
    Ray light_ray;
    light_ray.dir = -SunLightDir;
    light_ray.origin = hitpoint + sign(light_ray.dir) * hitbox.w * 0.01;
    float sun_light_factor = max(0.0, dot(normal, SunLightDir));
    if (sun_light_factor > 0.05) {
        // so that the angle between the light and the surface is not too small
        // when the angle is small, ray tracing it in the octree costs more
        if (!RayMarchTest(bounding_box, light_ray)) {
            // Not occluded
            // Add Sunlight
            light_color += sun_light_factor * SunLightColor.rgb;
        }
    }


    if (voxel_id == 0) {
        output_color = vec4(1.0, 1.0, 1.0, 1.0);
        diffuse_texture_id = 0;
    } else if ((voxel_id & 0x8000) == 0) {
        // regular
        uint material_id = voxel_id - 1;
        diffuse_texture_id = uint(regularMaterials[material_id].diffuse);
        output_color = vec4(1.0, 1.0, 1.0, 1.0);
        scale = regularMaterials[material_id].scale;
        output_color.xyz *= light_color;
    } else {
        // colored
        uint material_id = (voxel_id >> 8) & 0x7f;
        uint color = voxel_id & 0xff;
        diffuse_texture_id = uint(coloredMaterials[material_id].diffuse);
        output_color = coloredMaterials[material_id].palette[color];
        scale = coloredMaterials[material_id].scale;
        output_color.xyz *= light_color;
    }

    if (diffuse_texture_id > 0) {
        output_color *= texture(
            sampler2DArray(TextureRepo,  TextureRepoSampler),
            vec3(texcoords * scale, diffuse_texture_id-1)
        );
    }


    float ray_fog_factor = exp2(iteration * 18 - 18); // 0 for near, 1 for far
    f_color = output_color * (1 - ray_fog_factor);
    #endif
}
