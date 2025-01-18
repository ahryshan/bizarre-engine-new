#version 450

vec2 positions[] = {vec2(1.0, 1.0), vec2(-1.0, 1.0), vec2(-1.0, -1.0), vec2(1.0, -1.0)};
int indices[] = {0, 1, 2, 0, 2, 3};

layout(location = 0) out vec2 out_pos;

void main() {
     vec2 pos = positions[indices[gl_VertexIndex]];
     out_pos = pos;
     gl_Position = vec4(pos, 0.0, 1.0);
}