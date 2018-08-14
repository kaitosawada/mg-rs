#version 150 core
in vec3 position;
in vec3 normal;
out vec3 v_normal;
uniform mat4 u_model_view_proj;
void main() {
    v_normal = normal;
    gl_Position = u_model_view_proj * vec4(position, 1.0);
}
