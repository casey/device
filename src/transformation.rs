use super::*;

#[derive(Clone, Debug)]
#[allow(clippy::arbitrary_source_item_ordering)]
pub(crate) struct Transformation {
  pub(crate) space: Space,
  pub(crate) scaling: Vec3f,
  pub(crate) rotation: UnitQuaternion<f32>,
  pub(crate) translation: Vec3f,
}

impl Transformation {
  const SCALING_IDENTITY: Vec3f = Vec3f::new(1.0, 1.0, 1.0);

  pub(crate) fn response(&self, response: f32) -> Mat4f {
    let scaling = Self::SCALING_IDENTITY + (self.scaling - Self::SCALING_IDENTITY) * response;
    let rotation = self.rotation.powf(response);
    let translation = self.translation * response;

    let scaling = Mat4f::new_nonuniform_scaling(&scaling);
    let rotation = rotation.to_homogeneous();
    let translation = Translation3::from(translation).to_homogeneous();

    self.space.transform(translation * rotation * scaling)
  }
}

impl Default for Transformation {
  fn default() -> Self {
    Self {
      rotation: UnitQuaternion::identity(),
      scaling: Self::SCALING_IDENTITY,
      space: Space::Rgb,
      translation: Vec3f::zeros(),
    }
  }
}
