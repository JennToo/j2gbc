#version 150 core

uniform sampler2D t_Lcd;

in vec2 v_Uv;
out vec4 Target0;

void main() {
    Target0 = texture(t_Lcd, v_Uv);
}
