use super::*;

pub(crate) use {
  brown_noise::BrownNoise, cycle::Cycle, duty::Duty, envelope::Envelope, gain::Gain, gate::Gate,
  pink_noise::PinkNoise, saw::Saw, silence::Silence, sine::Sine, sum::Sum, white_noise::WhiteNoise,
};

mod brown_noise;
mod cycle;
mod duty;
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
  fn reset(&mut self);

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

  fn cycle(self, period: f32) -> Cycle<Self>
  where
    Self: Sized,
  {
    Cycle {
      inner: self,
      sample: 0,
      period: (period / 48_000.0) as u64,
    }
  }

  fn gain(self, gain: f32) -> Gain<Self>
  where
    Self: Sized,
  {
    Gain { inner: self, gain }
  }

  fn gate(self, after: f32) -> Gate<Self>
  where
    Self: Sized,
  {
    Gate {
      after,
      inner: self,
      timer: Timer::default(),
    }
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
