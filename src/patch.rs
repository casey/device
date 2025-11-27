use super::*;

#[derive(Clone, Copy, Default)]
pub(crate) enum Patch {
  Saw,
  #[default]
  Sine,
}

struct Wrapper<T: fundsp::audionode::AudioNode<Outputs = fundsp::prelude::U1>> {
  inner: fundsp::prelude::An<T>,
  sample: u64,
  samples: u64,
}

impl<T: fundsp::audionode::AudioNode<Outputs = fundsp::prelude::U1>> Source for Wrapper<T> {
  fn channels(&self) -> u16 {
    1
  }

  fn current_span_len(&self) -> Option<usize> {
    None
  }

  fn sample_rate(&self) -> u32 {
    44_100
  }

  fn total_duration(&self) -> Option<Duration> {
    None
  }
}

impl<T: fundsp::audionode::AudioNode<Outputs = fundsp::prelude::U1>> Iterator for Wrapper<T> {
  type Item = f32;

  fn next(&mut self) -> Option<f32> {
    if self.sample == self.samples {
      return None;
    }

    self.sample += 1;

    Some(self.inner.get_mono())
  }
}

fn adsr_one_shot(
  a: f32,
  d: f32,
  s: f32,
  r: f32,
  hold: f32,
) -> fundsp::prelude::An<impl fundsp::prelude::AudioNode<Outputs = fundsp::prelude::U1>> {
  let t_a = a;
  let t_d = a + d;
  let t_s_end = a + d + hold;
  let t_r_end = a + d + hold + r;

  fundsp::hacker32::envelope(move |t: f32| -> f32 {
    if t < 0.0 {
      0.0
    } else if t < t_a {
      // Attack: 0 -> 1
      t / a
    } else if t < t_d {
      // Decay: 1 -> s
      let u = (t - t_a) / d;
      1.0 + (s - 1.0) * u
    } else if t < t_s_end {
      // Sustain: s
      s
    } else if t < t_r_end {
      // Release: s -> 0
      let u = (t - t_s_end) / r;
      s * (1.0 - u)
    } else {
      // After release, stay at 0
      0.0
    }
  })
}

impl Patch {
  pub(crate) fn add(self, semitones: u8, tap: &Tap) {
    let frequency = 261.63 * 2.0f32.powf(semitones as f32 / 12.0);

    match self {
      Self::Sine => tap.add(Wrapper {
        inner: (fundsp::hacker32::sine_hz(frequency)
          * 0.25
          * adsr_one_shot(1.001, 0.1, 0.2, 0.1, 1.0)),
        // todo: convert to real value
        samples: 44_100,
        sample: 0,
      }),
      Self::Saw => tap.add(Wrapper {
        inner: (fundsp::hacker32::saw_hz(frequency)
          * 0.25
          * adsr_one_shot(1.001, 0.1, 0.2, 0.1, 1.0)),
        // todo: convert to real value
        samples: 44_100,
        sample: 0,
      }),
    }
  }
}
