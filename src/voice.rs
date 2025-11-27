use super::*;

pub(crate) use {
  brown_noise::BrownNoise, cycle::Cycle, envelope::Envelope, white_noise::WhiteNoise,
};

mod brown_noise;
mod cycle;
mod envelope;
mod white_noise;

pub(crate) trait Voice: Send {
  fn cycle(self, period: f32) -> Cycle<Self>
  where
    Self: Sized,
  {
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    Cycle {
      inner: self,
      sample: 0,
      period: (period * 48_000.0) as u64,
    }
  }

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
      timer: Timer::default(),
      attack,
      decay,
      sustain,
      release,
    }
  }

  fn reset(&mut self);

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
