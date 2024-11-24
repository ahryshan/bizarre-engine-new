#version 430

vec2 positions[3] = vec2[3](
     	vec2(-0.5, -0.5),
	vec2(0.0, 0.5),
	vec2(0.5, -0.5)
     );

layout(location = 0) out vec3 v_position;

void main() {
     vec2 position = positions[gl_VertexIndex];
     v_position = vec3(position, 0.0f);
     gl_Position = vec4(position, 0.0f, 1.0f);
}