#version 440

// Enterprise: the card's suit symbol in gold foil. A gradient of `gold`
// across it, lighter at the top left, and a band of light at `sweep` (which
// runs from -0.5 to 1.5 to cross it).

layout(location = 0) in vec2 qt_TexCoord0;
layout(location = 0) out vec4 fragColor;

layout(std140, binding = 0) uniform buf {
    mat4 qt_Matrix;
    float qt_Opacity;
    float sweep;
    vec4 gold;
};

layout(binding = 1) uniform sampler2D source;

void main() {
    vec4 c = texture(source, qt_TexCoord0);
    float d = (qt_TexCoord0.x + qt_TexCoord0.y) * 0.5;
    vec3 base = gold.rgb * mix(1.3, 0.65, d);
    float shine = 1.0 - smoothstep(0.0, 0.1, abs(d - sweep));
    vec3 rgb = min(base + shine * 0.55, vec3(1.0));
    // Premultiplied alpha: the glyph's coverage.
    fragColor = vec4(rgb * c.a, c.a) * qt_Opacity;
}
