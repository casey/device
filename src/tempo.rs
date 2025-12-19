use {super::*, std::process::Command};

static RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(\d+\.\d+) bpm\n$").unwrap());

#[derive(Clone, Copy)]
pub(crate) struct Tempo {
  pub(crate) bpm: f64,
  pub(crate) offset: f64,
}

impl Tempo {
  pub(crate) fn detect(path: &Utf8Path) -> Result<Self> {
    let bpm = {
      let stdout = Command::new("aubio").arg("tempo").arg(path).stdout_utf8()?;

      let captures = RE
        .captures(&stdout)
        .context(error::TempoParse { stdout: &stdout })?;

      captures[1]
        .parse::<f64>()
        .ok()
        .context(error::TempoParse { stdout: &stdout })?
    };

    let offset = {
      let stdout = Command::new("aubio").arg("beat").arg(path).stdout_utf8()?;

      let mut beats = Vec::new();

      for line in stdout.lines() {
        let line = line.trim();
        beats.push(
          line
            .parse::<f64>()
            .ok()
            .context(error::TempoParse { stdout: line })?,
        );
      }

      Self::detect_offset(bpm, &beats)
    };

    log::info!("loaded track with {bpm:.2} bpm at {offset:.2} offset");

    Ok(Self { bpm, offset })
  }

  fn detect_offset(bpm: f64, beats: &[f64]) -> f64 {
    let period = 60.0 / bpm;

    let mut best = None;

    let mut offset = 0.0;

    while offset < period {
      let mut error = 0.0;

      for &beat in beats {
        let phase = (beat - offset).rem_euclid(period);
        let distance = phase.min(period - phase);
        error += distance.min(0.03);
      }

      if best.is_none_or(|(_best_offset, best_error)| best_error > error) {
        best = Some((offset, error));
      }

      offset += 0.001;
    }

    best.map(|(offset, _error)| offset).unwrap()
  }
}

#[cfg(test)]
mod tests {
  use {super::*, assert_float_eq::assert_float_absolute_eq};

  #[test]
  fn detect_offset() {
    #[track_caller]
    fn case(bpm: f64, beats: &[f64], expected: f64) {
      assert_float_absolute_eq!(Tempo::detect_offset(bpm, beats), expected);
    }

    case(60.0, &[0.0, 1.0, 2.0, 3.0], 0.0);
    case(60.0, &[0.5, 1.5, 2.5, 3.5], 0.5);
    case(60.0, &[0.5, 1.0, 1.5, 2.5, 3.5], 0.5);
  }
}
