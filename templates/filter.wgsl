%% let mut binding = Counter::default();

@group(0)
@binding({{ binding.next() }})
var filtering_sampler: sampler;

@group(0)
@binding({{ binding.next() }})
var frequencies: texture_1d<f32>;

@group(0)
@binding({{ binding.next() }})
var source: texture_2d<f32>;

@group(0)
@binding({{ binding.next() }})
var non_filtering_sampler: sampler;

@group(0)
@binding({{ binding.next() }})
var samples: texture_1d<f32>;

@group(0)
@binding({{ binding.next() }})
var<uniform> uniforms: Uniforms;

const ERROR = vec4f(0, 1, 0, 1);
const TRANSPARENT = vec4f(0, 0, 0, 0);

%% for field in Field::iter() {
const {{ field.constant() }}: u32 = {{ field.number() }};
%% }

struct Uniforms {
  alpha: f32,
  base: f32,
  color: mat4x3f,
  coordinates: u32,
  destination_offset: vec2f,
  field: u32,
  frequency_range: f32,
  gain: f32,
  grid: f32,
  grid_alpha: f32,
  interpolate: u32,
  mirror: vec4f,
  parameter: f32,
  position: mat3x2f,
  repeat: u32,
  resolution: f32,
  response: f32,
  sample_range: f32,
  source_offset: vec2f,
  tiling: u32,
  wrap: u32,
}

fn coefficient() -> f32 {
  return uniforms.base + uniforms.response;
}

fn field_all(p: vec2f) -> bool {
  return true;
}

fn field_bottom(p: vec2f) -> bool {
  return field_top(-p);
}

fn field_circle(p: vec2f) -> bool {
  return length(p) < uniforms.parameter * coefficient();
}

fn field_cross(p: vec2f) -> bool {
  let x = abs(p.x);
  let y = abs(p.y);
  return min(x, y) < 0.25 * coefficient() && x < 1.0 && y < 1.0;
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
  return tan(radians(60)) * p.x - 0.5 * coefficient() < p.y
    && tan(radians(-60)) * p.x - 0.5  * coefficient() < p.y
    && p.y < 0.25 * coefficient();
}

fn field_x(p: vec2f) -> bool {
  let pixel = 2.0 / uniforms.resolution;
  return abs(abs(p.x) - abs(p.y)) < sqrt(2) * 0.25 * coefficient() - 0.5 * pixel;
}

fn grid(uv: vec2f) -> vec2f {
  return round(vec2(uv.x, (uv.y - 1) * -1)  * uniforms.grid)
    / uniforms.grid * uniforms.grid_alpha;
}

fn mod_floor(x: vec2f, y: f32) -> vec2f {
  return x - y * floor(x / y);
}

fn mirror(uv: vec2f) -> vec2f {
  let triangle = 1.0 - abs(2.0 * uv - 1.0);
  let inverse = 1.0 - triangle;
  let chosen = mix(triangle, inverse, uniforms.mirror.zw);
  return mix(uv, chosen, uniforms.mirror.xy);
}

@fragment
fn fragment(@builtin(position) position: vec4f) -> @location(0) vec4f {
  // subtract offset get tile coordinates
  let tile = position.xy - uniforms.destination_offset;

  // convert to uv coordinates
  let source_uv = tile / vec2(uniforms.resolution, uniforms.resolution);

  let mirrored_uv = mirror(source_uv);

  // convert tile coordinates to [-1, 1]
  let centered = mirrored_uv * 2 - 1;

  // apply position transform
  var transformed = uniforms.position * vec3(centered, 1);

  // convert position to uv coordinates
  var uv = (transformed + 1) / 2;

  // wrap transformed coordinates
  if bool(uniforms.wrap) {
    transformed = mod_floor(transformed + 1.0, 2.0) - 1.0;
  }

  // scale to compensate for tiles not taking up full front texture
  let tile_scale = vec2(uniforms.resolution, uniforms.resolution) * f32(uniforms.tiling)
    / vec2f(textureDimensions(source, 0));

  var input_color = TRANSPARENT;

  if bool(uniforms.repeat) || all(uv >= vec2(0.0, 0.0)) && all(uv <= vec2(1.0, 1.0)) {
    if bool(uniforms.coordinates) {
      input_color = vec4(uv, 1.0, 1.0);
    } else {
      // convert uv coordinates to tile source coordinates
      let tile_uv = (uv / f32(uniforms.tiling) + uniforms.source_offset) * tile_scale;

      // read input color
      if bool(uniforms.interpolate) {
        input_color = textureSample(source, filtering_sampler, tile_uv);
      } else {
        input_color = textureSample(source, non_filtering_sampler, tile_uv);
      }
    }
  }

  // Sample original color
  let original_color = textureSample(
    source,
    non_filtering_sampler,
    (mirrored_uv / f32(uniforms.tiling) + uniforms.source_offset) * tile_scale,
  );

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

  var alpha = 0.0;

  if on {
    alpha = uniforms.alpha;
  }

  // convert back to rgb
  var transformed_color = uniforms.color * vec4(input_color.rgb, 1.0);

  let grid = grid(mirrored_uv);
  transformed_color.g += grid.x;
  transformed_color.b += grid.y;

  // blend transformed and original color
  let blend = mix(original_color.rgb, transformed_color, alpha);

  // return blend with opaque alpha channel
  return vec4(blend, 1);
}
