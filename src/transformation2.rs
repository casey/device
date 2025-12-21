use super::*;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Transformation2 {
  pub(crate) period: Option<f32>,
  pub(crate) rotation: f32,
  pub(crate) scaling: Vec2f,
  pub(crate) sin: bool,
  pub(crate) translation: Vec2f,
}

impl Transformation2 {
  pub(crate) const SCALING_IDENTITY: Vec2f = Vec2f::new(1.0, 1.0);

  pub(crate) fn response(&self, response: f32) -> Mat3f {
    let response = self.period.map_or(response, |period| response % period);
    let response = if self.sin { response.sin() } else { response };

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
      period: None,
      rotation: 0.0,
      scaling: Self::SCALING_IDENTITY,
      sin: false,
      translation: Vec2f::zeros(),
    }
  }
}
