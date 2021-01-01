#version 450

layout(location=0) out vec4 f_color;
layout(location=0) in vec2 v_tex_coords;

void main() {
    f_color = vec4(v_tex_coords, 1.0, 1.0);
}
