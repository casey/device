use super::*;

#[derive(Clone, Debug)]
#[allow(clippy::arbitrary_source_item_ordering)]
pub(crate) struct Transformation2 {
  pub(crate) scaling: Vec2f,
  pub(crate) rotation: f32,
  pub(crate) translation: Vec2f,
  pub(crate) period: Option<f32>,
}

impl Transformation2 {
  const SCALING_IDENTITY: Vec2f = Vec2f::new(1.0, 1.0);

  pub(crate) fn response(&self, response: f32) -> Mat3f {
    let response = self.period.map_or(response, |period| response % period);
    let scaling = Self::SCALING_IDENTITY + (self.scaling - Self::SCALING_IDENTITY) * response;
    let rotation = self.rotation * response;
    let translation = self.translation * response;

    let scaling = Mat3f::new_nonuniform_scaling(&scaling);
    let rotation = Rotation2::new(rotation).to_homogeneous();
    let translation = Translation2::from(translation).to_homogeneous();

    translation * rotation * scaling
  }
}

impl Default for Transformation2 {
  fn default() -> Self {
    Self {
      rotation: 0.0,
      scaling: Self::SCALING_IDENTITY,
      translation: Vec2f::zeros(),
      period: None,
    }
  }
}
