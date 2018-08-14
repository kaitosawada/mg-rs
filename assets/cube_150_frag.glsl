#version 150 core
in vec3 v_normal;
out vec4 f_color;
const vec3 LIGHT = vec3(-0.5, 0.5, 0.5);
void main() {
    float diffuse  = clamp(dot(v_normal, LIGHT), 0.0, 1.0);
    vec4  light    = vec4(0.1, 0.3, 0.5, 1.0) * vec4(vec3(diffuse), 1.0);
    f_color         = light + 0.5;
}
