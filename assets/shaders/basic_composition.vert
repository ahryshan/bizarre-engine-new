#version 450

vec3 positions[] = {vec2(1.0, 1.0), vec2(-1.0, 1.0), vec2(-1.0, -1.0), vec2(1.0, -1.0)};
int indices[] = {0, 1, 2, 0, 2, 3};

void main() {
     vec2 pos = positions[indices[gl_VertexID]];
     gl_Position = vec4(pos, 0.0, 1.0);
}