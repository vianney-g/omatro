#version 440

// Overclocked: the card's suit symbol in a heat haze. The rows ripple
// sideways, more towards the top, as air does above something hot, and the
// symbol takes the `hot` colour from its bottom up. `strength` (0 to 1 and
// beyond as the card scores) sets how much; `time` runs from 0 to 2π.

layout(location = 0) in vec2 qt_TexCoord0;
layout(location = 0) out vec4 fragColor;

layout(std140, binding = 0) uniform buf {
    mat4 qt_Matrix;
    float qt_Opacity;
    float time;
    float strength;
    vec4 hot;
};

layout(binding = 1) uniform sampler2D source;

void main() {
    vec2 uv = qt_TexCoord0;
    float above = 1.0 - uv.y;
    float wave = 0.6 * sin(uv.y * 19.0 + time * 3.0)
               + 0.4 * sin(uv.y * 33.0 - time * 5.0 + 1.3);
    uv.x += wave * strength * (0.25 + 0.75 * above) * 0.035;
    uv.y += 0.4 * sin(uv.x * 23.0 + time * 4.0) * strength * 0.01;
    vec4 c = texture(source, uv);
    // Premultiplied alpha: the hot colour scaled by the glyph's coverage.
    float h = smoothstep(0.15, 0.95, uv.y) * min(strength, 1.0);
    c.rgb = mix(c.rgb, hot.rgb * c.a, h * 0.85);
    fragColor = c * qt_Opacity;
}
