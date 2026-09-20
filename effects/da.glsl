#version 100
precision mediump float;

varying vec2 uv;
varying vec4 color;

uniform sampler2D Texture;
uniform float u_time;
uniform float u_intensity;

// ПсевдоСЛУЧАЙНАЯ функция для генерации резких скачков (тремора)
float hash(vec2 p) {
    return fract(sin(dot(p, vec2(12.9898, 78.233))) * 43758.5453);
}

void main() {
    // --- 1. ВЫСОКОЧАСТОТНАЯ ВИБРАЦИЯ КАДРА (ТРЕВОЖНОСТЬ / ТРЕМОР) ---
    // Дробление времени для создания резкого дергания (около 45 FPS скачков)
    float jitter_time = floor(u_time * 45.0);
    vec2 jitter_dir = vec2(
        hash(vec2(jitter_time, 1.0)) - 0.5,
        hash(vec2(jitter_time, 2.0)) - 0.5
    );
    
    // Высокочастотные синусоиды для органической непрерывной микро-дрожи
    vec2 smooth_jitter = vec2(
        sin(u_time * 75.0),
        cos(u_time * 83.0)
    ) * 0.5;

    // Итоговый сдвиг UV-координат
    vec2 total_jitter = (jitter_dir * 0.6 + smooth_jitter * 0.4) * 0.0007 * u_intensity;
    vec2 sample_uv = uv + total_jitter;

    // --- 2. ХРОМАТИЧЕСКАЯ АБЕРРАЦИЯ ---
    vec2 center = sample_uv - vec2(0.6);
    float dist = length(center);

    float aberration_strength = dist * dist * 0.15 * u_intensity;
    vec2 offset = center * aberration_strength;

    float r = texture2D(Texture, sample_uv + offset).r;
    float g = texture2D(Texture, sample_uv).g;
    float b = texture2D(Texture, sample_uv - offset).b;
    float a = texture2D(Texture, sample_uv).a;

    vec3 col = vec3(r, g, b);

    // --- 3. ПУЛЬСИРУЮЩИЙ ПЕРЕСВЕТ (BLOOM/EXPOSURE PULSE) ---
    float pulse = 1.0 + (0.22 * sin(u_time * 4.0)) * u_intensity;
    col *= pulse;

    // --- 4. ВЫСОКАЯ КОНТРАСТНОСТЬ И ГИПЕРНАСЫЩЕННОСТЬ ---
    float contrast = mix(1.0, 1.55, u_intensity);
    col = (col - 0.5) * contrast + 0.4;

    float saturation = mix(1.0, 2.4, u_intensity);
    float luminance = dot(col, vec3(0.2126, 0.7152, 0.0722));
    col = mix(vec3(luminance), col, saturation);

    col = clamp(col, 0.0, 1.0);

    gl_FragColor = vec4(col, a) * color;
}
