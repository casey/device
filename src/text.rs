use super::*;

#[derive(Clone, Deserialize)]
pub(crate) struct Text {
  pub(crate) size: f32,
  pub(crate) string: String,
  pub(crate) x: f64,
  pub(crate) y: f64,
}
