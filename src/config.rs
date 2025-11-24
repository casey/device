use super::*;

#[derive(Default, Deserialize)]
pub(crate) struct Config {
  captures: Option<Utf8PathBuf>,
  music: Option<Utf8PathBuf>,
}

// todo:
// - increment if one already exists

impl Config {
  pub(crate) fn captures(&self) -> Option<&Utf8Path> {
    self.captures.as_deref()
  }

  pub(crate) fn load() -> Result<Self> {
    let path = home()?;

    let path = path.join(".config").join("device.yaml");

    let yaml = match fs::read_to_string(&path) {
      Err(err) => {
        if err.kind() == io::ErrorKind::NotFound {
          return Ok(Self::default());
        } else {
          return Err(error::FilesystemIo { path }.into_error(err));
        }
      }
      Ok(yaml) => yaml,
    };

    Ok(serde_yaml::from_str(&yaml).unwrap())
  }

  pub(crate) fn music(&self) -> Result<Utf8PathBuf> {
    if let Some(music) = &self.music {
      return Ok(music.into());
    }
    Ok(home()?.join("Music/Music/Media.localized/Music"))
  }
}
