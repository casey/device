use {
  serde::Deserialize,
  std::ops::{Add, AddAssign, SubAssign},
  strum::{EnumIter, IntoStaticStr},
};

pub use {field::Field, filter::Filter, parameter::Parameter, state::State, text::Text};

type Mat3f = nalgebra::Matrix3<f32>;
type Mat4f = nalgebra::Matrix4<f32>;
type Vec4f = nalgebra::Vector4<f32>;

pub fn invert_color() -> Mat4f {
  Mat4f::from_diagonal(&Vec4f::new(-1.0, -1.0, -1.0, 1.0))
}

mod field;
mod filter;
mod parameter;
mod state;
mod text;
