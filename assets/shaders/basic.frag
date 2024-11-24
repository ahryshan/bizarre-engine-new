#version 430

layout(location = 0) in vec3 v_position;

layout(location = 0) out vec4 f_color;

void main() {
     vec3 color = (v_position + 1.0) * 0.5;
     f_color = vec4(color, 1.0);
}