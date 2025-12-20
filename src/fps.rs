use super::*;

#[derive(Clone, Copy, Debug)]
pub(crate) struct Fps {
  dt: Duration,
  fps: NonZeroU32,
}

impl Fps {
  pub(crate) fn dt(self) -> Duration {
    self.dt
  }

  pub(crate) fn fps(self) -> NonZeroU32 {
    self.fps
  }

  pub(crate) fn spf(self, format: SoundFormat) -> Result<usize> {
    if !format.sample_rate.is_multiple_of(self.fps.get()) {
      return Err(
        error::SamplesPerFrame {
          fps: self,
          sample_rate: format.sample_rate,
        }
        .build(),
      );
    }

    Ok(format.sample_rate.into_usize() / self.fps.get().into_usize() * format.channels.into_usize())
  }
}

impl FromStr for Fps {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, String> {
    Ok(
      s.parse::<NonZeroU32>()
        .map_err(|err| format!("failed to parse fps: {err}"))?
        .into(),
    )
  }
}

impl From<NonZeroU32> for Fps {
  fn from(fps: NonZeroU32) -> Self {
    Self {
      dt: Duration::try_from_secs_f64(1.0 / fps.get() as f64).unwrap(),
      fps,
    }
  }
}

impl Display for Fps {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}", self.fps)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn new() {
    let _ = Fps::from(NonZeroU32::MIN);
    let _ = Fps::from(NonZeroU32::MAX);
  }
}
