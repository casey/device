use super::*;

#[derive(Default, Deserialize)]
pub(crate) struct Config {
  captures: Option<Utf8PathBuf>,
  music: Option<Utf8PathBuf>,
}

impl Config {
  pub(crate) fn captures(&self) -> Option<&Utf8Path> {
    self.captures.as_deref()
  }

  fn home() -> Result<Utf8PathBuf> {
    Ok(
      env::home_dir()
        .context(error::Home)?
        .into_utf8_path()?
        .to_owned(),
    )
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
