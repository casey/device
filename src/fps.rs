use super::*;

#[derive(Clone, Copy)]
pub(crate) struct Fps {
  pub(crate) duration: Duration,
}

impl FromStr for Fps {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, String> {
    let fps = s
      .parse::<f32>()
      .map_err(|err| format!("failed to parse fps: {err}"))?;

    let duration = Duration::try_from_secs_f32(1.0 / fps)
      .map_err(|err| format!("failed to calculate fps duration: {err}"))?;

    Ok(Self { duration })
  }
}
