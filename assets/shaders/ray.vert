#version 450

layout(location=0) out vec2 v_tex_coords;
layout(location=0) in vec2 position;

void main() {
    gl_Position = vec4(position, 0.0, 1.0);
    v_tex_coords = (position + 1.0) / 2.0;
}
