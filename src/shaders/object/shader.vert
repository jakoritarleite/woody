#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) in vec3 inPosition;

layout(binding = 0) uniform GlobalUniformObject {
    mat4 projection;
    mat4 view;
} gubo;

layout(push_constant) uniform PushConstants {
    mat4 model;
} pc;

void main() {
    gl_Position = gubo.projection * gubo.view * pc.model * vec4(inPosition, 1.0);
}
