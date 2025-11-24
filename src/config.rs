use super::*;

#[derive(Default, Deserialize)]
pub(crate) struct Config {
  captures: Option<Utf8PathBuf>,
  music: Option<Utf8PathBuf>,
}

impl Config {
  fn home() -> Result<Utf8PathBuf> {
    Ok(
      env::home_dir()
        .context(error::Home)?
        .into_utf8_path()?
        .to_owned(),
    )
  }

  pub(crate) fn capture(&self, extension: &str) -> Utf8PathBuf {
    let filename = format!(
      "{}.{extension}",
      SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs(),
    );

    self
      .captures
      .as_ref()
      .map(|captures| captures.join(&filename))
      .unwrap_or_else(|| filename.into())
  }

  pub(crate) fn load() -> Result<Self> {
    let path = Self::home()?.join(".config").join("device.yaml");

    let yaml = match fs::read_to_string(&path) {
      Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(Self::default()),
      Err(err) => return Err(error::FilesystemIo { path }.into_error(err)),
      Ok(yaml) => yaml,
    };

    serde_yaml::from_str(&yaml).context(error::ConfigDeserialize { path })
  }

  pub(crate) fn music(&self) -> Result<&Utf8Path> {
    self.music.as_deref().context(error::Music)
  }
}
