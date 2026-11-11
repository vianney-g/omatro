#version 440

// A joker's pixel art in the theme's colours: the drawing's grey is a palette
// index × 32 (build.rs), 0 transparent, each other index a colour of the theme.

layout(location = 0) in vec2 qt_TexCoord0;
layout(location = 0) out vec4 fragColor;

layout(std140, binding = 0) uniform buf {
    mat4 qt_Matrix;
    float qt_Opacity;
    vec4 foreground;
    vec4 accent;
    vec4 muted;
    vec4 background;
    vec4 warm;
    vec4 urgent;
    vec4 green;
};

layout(binding = 1) uniform sampler2D source;

void main() {
    int index = int(texture(source, qt_TexCoord0).r * 255.0 / 32.0 + 0.5);
    vec4 c = index == 1 ? foreground
           : index == 2 ? accent
           : index == 3 ? muted
           : index == 4 ? background
           : index == 5 ? warm
           : index == 6 ? urgent
           : index == 7 ? green
           : vec4(0.0);
    fragColor = vec4(c.rgb, 1.0) * c.a * qt_Opacity;
}
