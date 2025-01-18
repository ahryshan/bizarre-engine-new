#version 450
#extension GL_EXT_nonuniform_qualifier: enable

layout(location = 0) in vec3 in_position;
layout(location = 1) in vec3 in_normal;

layout(location = 0) out vec3 out_color;
layout(location = 1) out vec3 out_normal;
layout(location = 2) out vec3 out_position;

layout(set = 0, binding = 0) uniform SceneUniform {
    mat4 view;
    mat4 projection;
} scene_ubo;

layout(set = 0, binding = 1) uniform InstanceTransformsUbo {
    mat4 transform;
} instance_transforms_ubo[];

mat4 view = {
    {1.0, 0.0, -0.0, 0.0},
    {-0.0, 1.0, -0.0, 0.0},
    {0.0, 0.0, 1.0, 0.0},
    {0.0, -0.0, -10.0, 1.0}
};

mat4 projection = {
    {0.43301266, 0.0, 0.0, 0.0},
    {0.0, 0.57735026, 0.0, 0.0},
    {0.0, 0.0, -1.0001999, -1.0},
    {0.0, 0.0, -0.20002, 0.0}
};

void main() {
    mat4 instance_transform = instance_transforms_ubo[gl_InstanceIndex].transform;

    vec4 pos = scene_ubo.projection * scene_ubo.view * instance_transform * vec4(in_position, 1.0);
    gl_Position = pos;
    out_position = vec3(pos);

    out_color = vec3(1.0, 1.0, 1.0);
    out_normal = mat3(transpose(inverse(instance_transform))) * in_normal;
}
