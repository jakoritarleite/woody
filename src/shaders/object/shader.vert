#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) in vec3 inPosition;

layout(binding = 0) uniform GlobalUniformObject {
    mat4 projection;
    mat4 view;
} gubo;

void main() {
    gl_Position = gubo.projection * gubo.view * vec4(inPosition, 1.0);
}
