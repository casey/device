use super::*;

pub(crate) use {
  add::Add, brown_noise::BrownNoise, cycle::Cycle, duty::Duty, envelope::Envelope, gain::Gain,
  pink_noise::PinkNoise, saw::Saw, silence::Silence, sine::Sine, white_noise::WhiteNoise,
};

mod add;
mod brown_noise;
mod cycle;
mod duty;
mod envelope;
mod gain;
mod pink_noise;
mod saw;
mod silence;
mod sine;
mod white_noise;

pub(crate) trait Voice: Send {
  fn add<B: Voice + Sized>(self, b: B) -> Add<Self, B>
  where
    Self: Sized,
  {
    Add { a: self, b }
  }

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

  fn duty(self, on: f32, off: f32) -> Duty<Self>
  where
    Self: Sized,
  {
    Duty {
      inner: self,
      off,
      on,
      timer: Timer::default(),
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

  fn gain(self, gain: f32) -> Gain<Self>
  where
    Self: Sized,
  {
    Gain { inner: self, gain }
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
