#version 150 core
in vec3 position;
in vec3 normal;
out vec3 v_normal;
out vec3 v_view_position;
uniform mat4 model_view_matrix;
uniform mat4 projection_matrix;
uniform mat3 normal_matrix;
void main() {
    v_normal = normal_matrix * normal;
    vec4 mv_position = model_view_matrix * vec4(position, 1.0);
    gl_Position = projection_matrix * mv_position;
    v_view_position = -mv_position.xyz;
}
