use super::*;

pub(crate) use {
  brown_noise::BrownNoise, cycle::Cycle, envelope::Envelope, gain::Gain, gate::Gate,
  pink_noise::PinkNoise, saw::Saw, silence::Silence, sine::Sine, sum::Sum, white_noise::WhiteNoise,
};

mod brown_noise;
mod cycle;
mod envelope;
mod gain;
mod gate;
mod pink_noise;
mod saw;
mod silence;
mod sine;
mod sum;
mod white_noise;

pub(crate) trait Voice: Send {
  fn emitter(self) -> Emitter<Self>
  where
    Self: Sized,
  {
    Emitter::new(self)
  }

  fn envelope(self, attack: f32, decay: f32, sustain: f32, release: f32) -> Envelope<Self>
  where
    Self: Sized,
  {
    Envelope {
      inner: self,
      attack,
      decay,
      sustain,
      release,
    }
  }

  fn gain(self, gain: f32) -> Gain<Self>
  where
    Self: Sized,
  {
    Gain { inner: self, gain }
  }

  fn sample(&mut self) -> Option<f32>;

  fn source(self) -> Box<dyn Source + Send>
  where
    Self: Sized + 'static,
  {
    Box::new(self.emitter())
  }
}

fn distribution() -> Uniform<f32> {
  Uniform::new_inclusive(-1.0, 1.0).unwrap()
}
