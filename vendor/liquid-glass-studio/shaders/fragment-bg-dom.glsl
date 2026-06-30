#version 300 es

precision highp float;

in vec2 v_uv;
out vec4 fragColor;

uniform vec2 u_resolution;
uniform float u_dpr;
uniform vec2 u_mouseSpring;
uniform float u_mergeRate;
uniform float u_shapeWidth;
uniform float u_shapeHeight;
uniform float u_shapeRadius;
uniform float u_shapeRoundness;
uniform float u_shadowExpand;
uniform float u_shadowFactor;
uniform vec2 u_shadowPosition;
uniform int u_bgType;
uniform sampler2D u_bgTexture;
uniform int u_bgTextureReady;
uniform int u_showShape1;
uniform vec4 u_bgUVTransform;

#include './lib/sdf.glsl'

void main() {
  vec2 u_resolution1x = u_resolution.xy / u_dpr;
  vec3 bgColor = vec3(0.06, 0.06, 0.07);

  if (u_bgTextureReady == 1) {
    vec2 pageUV = u_bgUVTransform.xy + v_uv * u_bgUVTransform.zw;
    bgColor = texture(u_bgTexture, pageUV).rgb;
  }

  vec2 p1 =
    (vec2(0.0, 0.0) -
      u_resolution.xy * 0.5 +
      vec2(u_shadowPosition.x * u_dpr, u_shadowPosition.y * u_dpr)) /
    u_resolution.y;
  vec2 p2 =
    (vec2(0.0, 0.0) - u_mouseSpring + vec2(u_shadowPosition.x * u_dpr, u_shadowPosition.y * u_dpr)) /
    u_resolution.y;
  float merged = mainSDF(p1, p2, gl_FragCoord.xy);
  float shadow = exp(-1.0 / u_shadowExpand * abs(merged) * u_resolution1x.y) * 0.6 * u_shadowFactor;

  fragColor = vec4(bgColor - vec3(shadow), 1.0);
}
