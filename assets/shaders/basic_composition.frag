#version 450

layout(location = 0) in vec2 in_pos;

layout(input_attachment_index = 0, set = 0, binding = 0) uniform subpassInput inputColor;
layout(input_attachment_index = 1, set = 0, binding = 1) uniform subpassInput inputNormals;
layout(input_attachment_index = 2, set = 0, binding = 2) uniform subpassInput inputPositionDepth;

layout(location = 0) out vec4 out_color;


void main() {
    out_color = subpassLoad(inputColor);
    //out_color = vec4(0.5, 0.3, 1.0, 1.0);
}
