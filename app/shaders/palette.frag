#version 440

// A joker's pixel art in the theme's colours: the drawing's grey is a palette
// index × 8 (build.rs, ART_PALETTE), 0 transparent. Indices 1 to 10 are colours
// of the theme; 11 is ink, always dark, and 12 light, always light, whatever
// the theme (outlines and highlights keep their sense on light themes); 13 to
// 19 are dark shades of the theme's hues.

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
    vec4 cyan;
    vec4 magenta;
    vec4 orange;
};

layout(binding = 1) uniform sampler2D source;

// How far a shade goes towards black.
const float SHADE = 0.35;

float luma(vec4 c) { return dot(c.rgb, vec3(0.299, 0.587, 0.114)); }

void main() {
    int index = int(texture(source, qt_TexCoord0).r * 255.0 / 8.0 + 0.5);
    bool darkTheme = luma(background) < luma(foreground);
    vec4 darker = darkTheme ? background : foreground;
    vec4 lighter = darkTheme ? foreground : background;
    vec4 c = index == 1 ? foreground
           : index == 2 ? accent
           : index == 3 ? muted
           : index == 4 ? background
           : index == 5 ? warm
           : index == 6 ? urgent
           : index == 7 ? green
           : index == 8 ? cyan
           : index == 9 ? magenta
           : index == 10 ? orange
           : index == 11 ? mix(darker, vec4(0.0, 0.0, 0.0, 1.0), 0.4)
           : index == 12 ? mix(lighter, vec4(1.0), 0.4)
           : index == 13 ? mix(accent, vec4(0.0, 0.0, 0.0, 1.0), SHADE)
           : index == 14 ? mix(warm, vec4(0.0, 0.0, 0.0, 1.0), SHADE)
           : index == 15 ? mix(urgent, vec4(0.0, 0.0, 0.0, 1.0), SHADE)
           : index == 16 ? mix(green, vec4(0.0, 0.0, 0.0, 1.0), SHADE)
           : index == 17 ? mix(cyan, vec4(0.0, 0.0, 0.0, 1.0), SHADE)
           : index == 18 ? mix(magenta, vec4(0.0, 0.0, 0.0, 1.0), SHADE)
           : index == 19 ? mix(orange, vec4(0.0, 0.0, 0.0, 1.0), SHADE)
           : vec4(0.0);
    float a = index == 0 ? 0.0 : 1.0;
    fragColor = vec4(c.rgb, 1.0) * a * qt_Opacity;
}
