#version 430

layout(location = 0) in vec3 in_color;
layout(location = 1) in vec3 in_normal;
layout(location = 2) in vec3 in_position;

layout(location = 0) out vec4 out_color;
layout(location = 1) out vec4 out_normal;
layout(location = 2) out vec4 out_position;

void main() {
     out_color = vec4(in_color, 1.0);
     out_normal = vec4(in_normal, 0.0);
     out_position = vec4(in_position, 0.0);
}