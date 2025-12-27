use super::*;

#[derive(Default, Deserialize)]
pub(crate) struct Config {
  captures: Option<Utf8PathBuf>,
  images: Option<Utf8PathBuf>,
  music: Option<Utf8PathBuf>,
}

impl Config {
  pub(crate) fn capture(&self, stem: Option<&str>, extension: &str) -> Utf8PathBuf {
    let stem = match stem {
      Some(stem) => stem.into(),
      None => SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        .to_string(),
    };

    let filename = format!("{stem}.{extension}");

    if let Some(captures) = &self.captures {
      captures.join(filename)
    } else {
      filename.into()
    }
  }

  fn find(dir: &Utf8Path, pattern: &str) -> Result<Utf8PathBuf> {
    let regex = RegexBuilder::new(pattern)
      .case_insensitive(true)
      .build()
      .context(error::FindRegex)?;

    let mut matches = Vec::<Utf8PathBuf>::new();

    for entry in WalkDir::new(dir) {
      let entry = entry.context(error::FindWalk)?;

      if entry.file_type().is_dir() {
        continue;
      }

      let path = entry.path();

      let haystack = path.strip_prefix(dir).unwrap().with_extension("");

      let Some(haystack) = haystack.to_str() else {
        continue;
      };

      if regex.is_match(haystack) {
        matches.push(path.into_utf8_path()?.into());
      }
    }

    if matches.len() > 1 {
      return Err(error::FindAmbiguous { matches }.build());
    }

    match matches.into_iter().next() {
      Some(path) => Ok(path),
      None => Err(error::FindMatch { pattern }.build()),
    }
  }

  pub(crate) fn find_image(&self, pattern: &str) -> Result<Utf8PathBuf> {
    Self::find(self.images()?, pattern)
  }

  pub(crate) fn find_song(&self, pattern: &str) -> Result<Utf8PathBuf> {
    Self::find(self.music()?, pattern)
  }

  fn home() -> Result<Utf8PathBuf> {
    Ok(
      env::home_dir()
        .context(error::Home)?
        .into_utf8_path()?
        .to_owned(),
    )
  }

  pub(crate) fn images(&self) -> Result<&Utf8Path> {
    self.images.as_deref().context(error::Images)
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
