#version 450

layout(location = 0) in vec3 in_position;
layout(location = 1) in vec3 in_normal;

layout(location = 0) out vec4 out_color;
layout(location = 1) out vec4 out_normal;
layout(location = 2) out vec4 out_position_depth;

void main() {
     out_color = vec4(1.0, 1.0, 1.0, 1.0);
     out_normal = vec4(1.0, 0.0, 0.0, 0.0);
     out_position_depth = vec4(0.0, 1.0, 0.0, 0.0);
}