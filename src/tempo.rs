use {super::*, std::process::Command};

#[derive(Clone, Copy)]
pub(crate) struct Tempo {
  pub(crate) bpm: f64,
  pub(crate) offset: f64,
}

const RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(\d+\.\d+) bpm\n$").unwrap());

impl Tempo {
  pub(crate) fn load(path: &Utf8Path, offset: f64) -> Result<Self> {
    let stdout = Command::new("aubio").arg("tempo").arg(path).stdout_utf8()?;

    let captures = RE
      .captures(&stdout)
      .context(error::TempoParse { stdout: &stdout })?;

    let bpm = captures[1]
      .parse::<f64>()
      .context(error::TempoBpm { stdout })?;

    Ok(Self { bpm, offset })
  }
}
