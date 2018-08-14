#version 150 core
in vec3 v_normal;
in vec2 v_texture;
in vec3 v_view_position;
out vec4 f_color;

// uniforms
uniform float metallic;
uniform float roughness;
uniform vec3 albedo;
uniform int has_kdmap;
uniform vec3 emissive;
uniform float opacity;

uniform sampler2D t_color;

// defines
#define PI 3.14159265359
#define PI2 6.28318530718
#define RECIPROCAL_PI 0.31830988618
#define RECIPROCAL_PI2 0.15915494
#define LOG2 1.442695
#define EPSILON 1e-6

struct IncidentLight {
  vec3 color;
  vec3 direction;
  bool visible;
};

struct ReflectedLight {
  vec3 directDiffuse;
  vec3 directSpecular;
  vec3 indirectDiffuse;
  vec3 indirectSpecular;
};

struct GeometricContext {
  vec3 position;
  vec3 normal;
  vec3 viewDir;
};

struct Material {
  vec3 diffuseColor;
  float specularRoughness;
  vec3 specularColor;
};

// lights

bool testLightInRange(const in float lightDistance, const in float cutoffDistance) {
  return any(bvec2(cutoffDistance == 0.0, lightDistance < cutoffDistance));
}

float punctualLightIntensityToIrradianceFactor(const in float lightDistance, const in float cutoffDistance, const in float decayExponent) {
  if (decayExponent > 0.0) {
    return pow(clamp(-lightDistance / cutoffDistance + 1.0, 0.0, 1.0), decayExponent);
  }

  return 1.0;
}

struct DirectionalLight {
  vec3 color;
  vec3 direction;
};

void getDirectionalDirectLightIrradiance(const in DirectionalLight directionalLight, const in GeometricContext geometry, out IncidentLight directLight) {
  directLight.color = directionalLight.color;
  directLight.direction = directionalLight.direction;
  directLight.visible = true;
}

struct PointLight {
  vec3 position;
  vec3 color;
  vec3 distance_decay;
  //float decay;
};

void getPointDirectLightIrradiance(const in PointLight pointLight, const in GeometricContext geometry, out IncidentLight directLight) {
  vec3 L = pointLight.position - geometry.position;
  directLight.direction = normalize(L);

  float lightDistance = length(L);
  if (testLightInRange(lightDistance, pointLight.distance_decay.x)) {
    directLight.color = pointLight.color;
    directLight.color *= punctualLightIntensityToIrradianceFactor(lightDistance, pointLight.distance_decay.x, pointLight.distance_decay.y);
    directLight.visible = true;
  } else {
    directLight.color = vec3(0.0);
    directLight.visible = false;
  }
}

struct SpotLight {
  vec3 position;
  vec3 direction;
  vec3 color;
  vec4 distance_decay_coneCos_penumbraCos;
};

void getSpotDirectLightIrradiance(const in SpotLight spotLight, const in GeometricContext geometry, out IncidentLight directLight) {
  vec3 L = spotLight.position - geometry.position;
  directLight.direction = normalize(L);

  float lightDistance = length(L);
  float angleCos = dot(directLight.direction, spotLight.direction);

  if (all(bvec2(angleCos > spotLight.distance_decay_coneCos_penumbraCos.z, testLightInRange(lightDistance, spotLight.distance_decay_coneCos_penumbraCos.x)))) {
    float spotEffect = smoothstep(spotLight.distance_decay_coneCos_penumbraCos.z, spotLight.distance_decay_coneCos_penumbraCos.w, angleCos);
    directLight.color = spotLight.color;
    directLight.color *= spotEffect * punctualLightIntensityToIrradianceFactor(lightDistance, spotLight.distance_decay_coneCos_penumbraCos.x, spotLight.distance_decay_coneCos_penumbraCos.y);
    directLight.visible = true;
  } else {
    directLight.color = vec3(0.0);
    directLight.visible = false;
  }
}

// light uniforms
#define LIGHT_MAX 4
uniform d_lights {
    DirectionalLight directionalLights[LIGHT_MAX];
};
uniform p_lights {
    PointLight pointLights[LIGHT_MAX];
};
uniform s_lights {
    SpotLight spotLights[LIGHT_MAX];
};
uniform int numDirectionalLights;
uniform int numPointLights;
uniform int numSpotLights;

// BRDFs

// Normalized Lambert
vec3 DiffuseBRDF(vec3 diffuseColor) {
  return diffuseColor / PI;
}

vec3 F_Schlick(vec3 specularColor, vec3 H, vec3 V) {
  return (specularColor + (1.0 - specularColor) * pow(1.0 - clamp(dot(V,H), 0.0, 1.0), 5.0));
}

float D_GGX(float a, float dotNH) {
  float a2 = a*a;
  float dotNH2 = dotNH*dotNH;
  float d = dotNH2 * (a2 - 1.0) + 1.0;
  return a2 / (PI * d * d);
}

float G_Smith_Schlick_GGX(float a, float dotNV, float dotNL) {
  float k = a*a*0.5 + EPSILON;
  float gl = dotNL / (dotNL * (1.0 - k) + k);
  float gv = dotNV / (dotNV * (1.0 - k) + k);
  return gl*gv;
}

// Cook-Torrance
vec3 SpecularBRDF(const in IncidentLight directLight, const in GeometricContext geometry, vec3 specularColor, float roughnessFactor) {

  vec3 N = -geometry.normal;
  vec3 V = geometry.viewDir;
  vec3 L = -directLight.direction;

  float dotNL = clamp(dot(N,L), 0.0, 1.0);
  float dotNV = clamp((dot(N,V)), 0.0, 1.0);
  vec3 H = normalize(L+V);
  float dotNH = clamp((dot(N,H)), 0.0, 1.0);
  float dotVH = clamp((dot(V,H)), 0.0, 1.0);
  float dotLV = clamp((dot(L,V)), 0.0, 1.0);
  float a = roughnessFactor * roughnessFactor;

  float D = D_GGX(a, dotNH);
  float G = G_Smith_Schlick_GGX(a, dotNV, dotNL);
  vec3 F = F_Schlick(specularColor, V, H);
  return (F*(G*D))/(4.0*dotNL*dotNV+EPSILON);
}

// RenderEquations(RE)
void RE_Direct(const in IncidentLight directLight, const in GeometricContext geometry, const in Material material, inout ReflectedLight reflectedLight) {

  float dotNL = clamp((dot(geometry.normal, directLight.direction)), 0.0, 1.0);
  vec3 irradiance = dotNL * directLight.color;

  // punctual light
  irradiance *= PI;

  reflectedLight.directDiffuse += irradiance * DiffuseBRDF(material.diffuseColor);
  reflectedLight.directSpecular += irradiance * SpecularBRDF(directLight, geometry, material.specularColor, material.specularRoughness);
}

void main() {
  vec3 aw;
  if (has_kdmap != 0) {
    aw = texture(t_color, v_texture).rgb;
  } else {
    aw = albedo;
  }
  GeometricContext geometry;
  geometry.position = -v_view_position;
  geometry.normal = normalize(v_normal);
  geometry.viewDir = normalize(v_view_position);

  Material material;
  material.diffuseColor = mix(aw, vec3(0.0), metallic);
  material.specularColor = mix(vec3(0.04), aw, metallic);
  material.specularRoughness = roughness;

  // Lighting

  ReflectedLight reflectedLight = ReflectedLight(vec3(0.0), vec3(0.0), vec3(0.0), vec3(0.0));

  IncidentLight directLight;

  // point light
  for (int i=0; i<LIGHT_MAX; ++i) {
    if (i >= numPointLights) break;
    getPointDirectLightIrradiance(pointLights[i], geometry, directLight);
    if (directLight.visible) {
      RE_Direct(directLight, geometry, material, reflectedLight);
    }
  }

  // spot light
  for (int i=0; i<LIGHT_MAX; ++i) {
    if (i >= numSpotLights) break;
    getSpotDirectLightIrradiance(spotLights[i], geometry, directLight);
    if (directLight.visible) {
      RE_Direct(directLight, geometry, material, reflectedLight);
    }
  }

  // directional light
  for (int i=0; i<LIGHT_MAX; ++i) {
    if (i >= numDirectionalLights) break;
    getDirectionalDirectLightIrradiance(directionalLights[i], geometry, directLight);
    RE_Direct(directLight, geometry, material, reflectedLight);
  }

  vec3 outgoingLight = emissive + reflectedLight.directDiffuse + reflectedLight.directSpecular + reflectedLight.indirectDiffuse + reflectedLight.indirectSpecular;

  f_color = vec4(outgoingLight, opacity);
}