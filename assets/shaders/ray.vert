#version 450

layout(location=0) out vec2 v_tex_coords;


const vec2 positions[4] = vec2[4](
vec2(-1.0, 1.0),
vec2(1.0, 1.0),
vec2(-1.0, -1.0),
vec2(1.0, -1.0)
);

void main() {
    vec2 position = positions[gl_VertexIndex];
    gl_Position = vec4(position, 0.5, 1.0);
    v_tex_coords = (position + 1.0) / 2.0;
}
