use super::*;

pub(crate) use {
  brown_noise::BrownNoise, cycle::Cycle, envelope::Envelope, gain::Gain, gate::Gate,
  pink_noise::PinkNoise, silence::Silence, sine::Sine, sum::Sum, white_noise::WhiteNoise,
};

// todo:
// - experiment with brown noise gain
//
// - better envelope tests:
//   try different lengths of phases
//
// - seed different rngs differently

mod brown_noise;
mod cycle;
mod envelope;
mod gain;
mod gate;
mod pink_noise;
mod silence;
mod sine;
mod sum;
mod white_noise;

pub(crate) trait Voice: Send {
  fn gain(self, gain: f32) -> Gain<Self>
  where
    Self: Sized,
  {
    Gain { inner: self, gain }
  }

  fn sample(&mut self, t: f32) -> f32;
}

fn distribution() -> Uniform<f32> {
  Uniform::new_inclusive(-1.0, 1.0).unwrap()
}
