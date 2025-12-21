use super::*;

#[derive(Clone, Debug)]
pub(crate) struct Transformation3 {
  pub(crate) period: Option<f32>,
  pub(crate) rotation: UnitQuaternion<f32>,
  pub(crate) scaling: Vec3f,
  pub(crate) sin: bool,
  pub(crate) space: Space,
  pub(crate) translation: Vec3f,
}

impl Transformation3 {
  const SCALING_IDENTITY: Vec3f = Vec3f::new(1.0, 1.0, 1.0);

  pub(crate) fn response(&self, response: f32) -> Mat4f {
    let response = self.period.map_or(response, |period| response % period);
    let response = if self.sin { response.sin() } else { response };

    let scaling = Self::SCALING_IDENTITY + (self.scaling - Self::SCALING_IDENTITY) * response;
    let rotation = self.rotation.powf(response);
    let translation = self.translation * response;

    let scaling = Mat4f::new_nonuniform_scaling(&scaling);
    let rotation = rotation.to_homogeneous();
    let translation = Translation3::from(translation).to_homogeneous();

    self.space.transform(translation * rotation * scaling)
  }
}

impl Default for Transformation3 {
  fn default() -> Self {
    Self {
      period: None,
      rotation: UnitQuaternion::identity(),
      scaling: Self::SCALING_IDENTITY,
      sin: false,
      space: Space::Rgb,
      translation: Vec3f::zeros(),
    }
  }
}
