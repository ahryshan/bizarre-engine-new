#version 450

#define MAX_UNIFORM_LENGTH 1024

layout(location = 0) in vec3 in_position;
layout(location = 1) in vec3 in_normal;

layout(location = 0) out vec3 out_color;
layout(location = 1) out vec3 out_normal;
layout(location = 2) out vec3 out_position;

layout(set = 0, binding = 0) uniform SceneUniform {
    mat4 view;
    mat4 projection;
} scene_ubo;

struct InstanceData {
    mat4 transform;
    vec3 color;
} instance_data;

layout(set = 1, binding = 0) uniform InstanceUbo {
    InstanceData data[MAX_UNIFORM_LENGTH];
} instance_ubo;

void main() {
    InstanceData instance_data = instance_ubo.data[gl_InstanceIndex];

    vec4 pos = scene_ubo.projection * scene_ubo.view * instance_data.transform * vec4(in_position, 1.0);
    gl_Position = pos;
    out_position = vec3(pos);

    out_color = instance_data.color;
    // out_color = vec3(1);
    out_normal = mat3(transpose(inverse(instance_data.transform))) * in_normal;
}
