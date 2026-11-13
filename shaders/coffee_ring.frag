#version 460 core

// A dried coffee ring, as a mug leaves it: the pigment piles up at the rim
// (a sharp outer edge that fades inward), the rim is patchy along its
// sweep and wobbles off the circle, a faint tide tints the inside, and a
// second, fainter ring sits a little off where the mug was set down again.
// Drawn once into the label's snapshot, not per frame.

#include <flutter/runtime_effect.glsl>

uniform vec2 uCenter; // px
uniform float uRadius; // px
uniform vec4 uColor; // straight rgb, a = the darkest the rim gets
uniform float uSeed;

out vec4 fragColor;

float hash(vec2 p) {
  p = fract(p * vec2(123.34, 456.21));
  p += dot(p, p + 45.32);
  return fract(p.x * p.y);
}

float noise(vec2 p) {
  vec2 i = floor(p);
  vec2 f = fract(p);
  vec2 u = f * f * (3.0 - 2.0 * f);
  return mix(
    mix(hash(i), hash(i + vec2(1.0, 0.0)), u.x),
    mix(hash(i + vec2(0.0, 1.0)), hash(i + vec2(1.0, 1.0)), u.x),
    u.y
  );
}

float fbm(vec2 p) {
  float sum = 0.0;
  float amplitude = 0.5;
  for (int i = 0; i < 4; i++) {
    sum += amplitude * noise(p);
    p *= 2.03;
    amplitude *= 0.5;
  }
  return sum;
}

// How much pigment one ring leaves at `p` (ring radius 1), `px` being one
// pixel in those units.
float ring(vec2 p, float px, float seed) {
  float r = length(p);
  // Sampled on the unit circle, so the rim has no seam.
  vec2 dir = p / max(r, 1e-4);
  float wobble = (fbm(dir * 1.3 + seed) - 0.5) * 0.035 +
    (noise(dir * 9.0 + seed * 1.7) - 0.5) * 0.012;
  float d = r - (1.0 + wobble); // < 0 inside
  float inside = 1.0 - smoothstep(-px, px, d);
  // A thin crisp line where the coffee dried last, then a fainter band.
  float width = mix(0.008, 0.028, fbm(dir * 2.3 + seed + 5.0));
  float line = exp(min(d, 0.0) / width);
  float band = 0.3 * exp(min(d, 0.0) / 0.12);
  float rim = inside * (line + band);
  float patchy = smoothstep(0.25, 0.7, fbm(dir * 1.2 + seed + 11.0));
  float tide = inside * 0.1 * smoothstep(0.35, 0.75, fbm(p * 4.0 + seed + 23.0));
  return rim * mix(0.15, 1.0, patchy) + tide;
}

void main() {
  vec2 frag = FlutterFragCoord().xy;
  vec2 p = (frag - uCenter) / uRadius;
  float px = 1.5 / uRadius;
  float a = ring(p, px, uSeed);
  a = max(a, 0.45 * ring((p - vec2(0.07, -0.05)) * 1.04, px, uSeed + 31.0));
  // The paper takes the pigment unevenly.
  a *= 0.75 + 0.25 * noise(frag * 0.9);
  a = clamp(a, 0.0, 1.0) * uColor.a;
  fragColor = vec4(uColor.rgb * a, a);
}
