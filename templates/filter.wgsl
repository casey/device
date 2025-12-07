%% let mut next = 0;
%% let mut binding = || {
%%   let binding = next;
%%   next += 1;
%%   binding
%% };

@group(0)
@binding({{ binding() }})
var filtering_sampler: sampler;

@group(0)
@binding({{ binding() }})
var frequencies: texture_1d<f32>;

@group(0)
@binding({{ binding() }})
var input: texture_2d<f32>;

@group(0)
@binding({{ binding() }})
var non_filtering_sampler: sampler;

@group(0)
@binding({{ binding() }})
var samples: texture_1d<f32>;

@group(0)
@binding({{ binding() }})
var<uniform> uniforms: Uniforms;

const ERROR = vec4f(0, 1, 0, 1);
const TRANSPARENT = vec4f(0, 0, 0, 0);

%% for field in Field::iter() {
const {{ field.constant() }}: u32 = {{ field.number() }};
%% }

struct Uniforms {
  color: mat4x4f,
  coordinates: u32,
  field: u32,
  frequency_range: f32,
  front_offset: vec2f,
  gain: f32,
  interpolate: u32,
  offset: vec2f,
  position: mat3x3f,
  repeat: u32,
  resolution: f32,
  rms: f32,
  sample_range: f32,
  tiling: u32,
  wrap: u32,
}

fn coefficient() -> f32 {
  return 1 + uniforms.rms / 10 * uniforms.gain;
}

fn field_all(p: vec2f) -> bool {
  return true;
}

fn field_bottom(p: vec2f) -> bool {
  return field_top(-p);
}

fn field_circle(p: vec2f) -> bool {
  return length(p) < 0.5 * coefficient();
}

fn field_cross(p: vec2f) -> bool {
  return min(abs(p.x), abs(p.y)) < 0.25 * coefficient();
}

fn field_frequencies(p: vec2f) -> bool {
  let x = (p.x + 1) * 0.5 * uniforms.frequency_range;
  let level = textureSample(frequencies, non_filtering_sampler, x).x * uniforms.gain;
  return level > (-p.y + 1) * 0.5;
}

fn field_left(p: vec2f) -> bool {
  return p.x + 1 < coefficient();
}

fn field_none(p: vec2f) -> bool {
  return false;
}

fn field_right(p: vec2f) -> bool {
  return field_left(-p);
}

fn field_samples(p: vec2f) -> bool {
  let x = (p.x + 1) * 0.5 * uniforms.sample_range;
  let level = textureSample(samples, non_filtering_sampler, x).x * uniforms.gain;
  return level < p.y;
}

fn field_square(p: vec2f) -> bool {
  return max(abs(p.x), abs(p.y)) < 0.5 * coefficient();
}

fn field_top(p: vec2f) -> bool {
  return p.y + 1 < coefficient();
}

fn field_triangle(p: vec2f) -> bool {
  return tan(radians(60)) * p.x - 0.5 < p.y
    && tan(radians(-60)) * p.x - 0.5 < p.y
    && p.y < 0.25;
}

fn field_x(p: vec2f) -> bool {
  let pixel = 2.0 / uniforms.resolution;
  return abs(abs(p.x) - abs(p.y)) < sqrt(2) * 0.25 * coefficient() - 0.5 * pixel;
}

fn read(uv: vec2f) -> bool {
  return bool(uniforms.repeat) || all(uv >= vec2(0.0, 0.0)) && all(uv <= vec2(1.0, 1.0));
}

fn sample(texture: texture_2d<f32>, uv: vec2f) -> vec4f {
  if bool(uniforms.interpolate) {
    return textureSample(texture, filtering_sampler, uv);
  } else {
    return textureSample(texture, non_filtering_sampler, uv);
  }
}

@fragment
fn fragment(@builtin(position) position: vec4f) -> @location(0) vec4f {
  // subtract offset get tile coordinates
  let tile = position.xy - uniforms.offset;

  // convert tile coordinates to [-1, 1]
  let centered = tile / vec2(uniforms.resolution, uniforms.resolution) * 2 - 1;

  // apply position transform
  let transformed = (uniforms.position * vec3(centered, 1)).xy;

  // convert position to uv coordinates
  var uv = (transformed + 1) / 2;

  // wrap uv coordinates
  if bool(uniforms.wrap) {
    uv = fract(uv);
  }

  var input_color = TRANSPARENT;

  if read(uv) {
    if bool(uniforms.coordinates) {
      input_color = vec4(uv, 1.0, 1.0);
    } else {
      // convert uv coordinates to tile source coordinates
      var tile_uv = uv / f32(uniforms.tiling) + uniforms.front_offset;

      // scale to compensate for tiles not taking up full front texture
      let scale = vec2(uniforms.resolution, uniforms.resolution) * f32(uniforms.tiling)
        / vec2f(textureDimensions(input, 0));
      tile_uv *= scale;

      // read input color
      input_color = sample(input, tile_uv);
    }
  }

  let input = vec4(input_color.rgb, 1.0);

  var on: bool;

  switch uniforms.field {
%% for field in Field::iter() {
    case {{ field.constant() }} {
      on = {{ field.function() }}(transformed);
    }
%% }
    default {
      return ERROR;
    }
  }

  if on {
    // convert rgb color to [-1, 1]
    let centered = input * 2 - 1;

    // apply color transform
    let transformed = uniforms.color * centered;

    // convert back to rgb
    let color = (transformed + 1) / 2;

    return color;
  } else {
    return input;
  }
}
