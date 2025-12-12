%% let mut binding = Counter::default();

@group(0)
@binding({{ binding.next() }})
var destination: texture_2d<f32>;

@group(0)
@binding({{ binding.next() }})
var source: texture_2d<f32>;

@group(0)
@binding({{ binding.next() }})
var texture_sampler: sampler;

@group(0)
@binding({{ binding.next() }})
var<uniform> uniforms: Uniforms;

struct Uniforms {
  destination: u32,
  source: u32,
  viewport: mat3x2f,
}

fn sample(condition: u32, texture: texture_2d<f32>, uv: vec2f) -> vec4f {
  if bool(condition) {
    return textureSample(texture, texture_sampler, uv);
  } else {
    return vec4f(0, 0, 0, 0);
  }
}

@fragment
fn fragment(@builtin(position) position: vec4f) -> @location(0) vec4f {
  let uv = uniforms.viewport * vec3(position.xy, 1.0);

  let src = sample(uniforms.source, source, uv);

  let dst = sample(uniforms.destination, destination, uv);

  let blend = mix(dst.rgb, src.rgb, src.a);

  return vec4(blend, 1);
}
