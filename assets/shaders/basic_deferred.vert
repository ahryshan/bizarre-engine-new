#version 430

layout(location = 0) in vec3 in_position;

layout(location = 0) out vec3 out_position;
layout(location = 1) out vec3 out_normal;

void main() {
    gl_Position = vec4(in_position, 1.0);
    out_position = in_position;
}
