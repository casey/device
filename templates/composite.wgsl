%% let mut next = 0;
%% let mut binding = || {
%%   let binding = next;
%%   next += 1;
%%   binding
%% };

@group(0)
@binding({{ binding() }})
var back: texture_2d<f32>;

@group(0)
@binding({{ binding() }})
var front: texture_2d<f32>;

@group(0)
@binding({{ binding() }})
var texture_sampler: sampler;

@group(0)
@binding({{ binding() }})
var<uniform> uniforms: Uniforms;

const TRANSPARENT = vec4f(0, 0, 0, 0);

struct Uniforms {
  back_read: u32,
  fit: u32,
  front_read: u32,
  resolution: vec2f,
}

@fragment
fn fragment(@builtin(position) position: vec4f) -> @location(0) vec4f {
  // convert fragment coordinates to [-1, 1]
  var centered = position.xy / uniforms.resolution * 2 - 1;

  // calculate aspect ratio
  let aspect = uniforms.resolution.x / uniforms.resolution.y;

  if bool(uniforms.fit) {
    // fit to viewport
    if aspect > 1 {
      centered.x *= aspect;
    } else {
      centered.y /= aspect;
    }
  } else {
    // fill viewport
    if aspect > 1 {
      centered.y /= aspect;
    } else {
      centered.x *= aspect;
    }
  }

  // convert to uv coordinates
  let uv = (centered + 1) / 2;

  var front_color = TRANSPARENT;

  if bool(uniforms.front_read) {
    // read front color
    front_color = textureSample(front, texture_sampler, uv);
  }

  var back_color = TRANSPARENT;

  if bool(uniforms.back_read) {
    // read back color
    back_color = textureSample(back, texture_sampler, uv);
  }

  let blend = front_color.rgb * front_color.a + back_color.rgb * (1 - front_color.a);

  return vec4(blend, 1.0);
}
