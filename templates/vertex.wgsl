const VERTICES = array(
  vec2f(-1, -1),
  vec2f(-1, 3),
  vec2f(3, -1),
);

@vertex
fn vertex(@builtin(vertex_index) i: u32) -> @builtin(position) vec4f {
  return vec4(VERTICES[i], 0, 1);
}
