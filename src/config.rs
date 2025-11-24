use super::*;

#[derive(Default, Deserialize)]
pub(crate) struct Config {
  captures: Option<Utf8PathBuf>,
  music: Option<Utf8PathBuf>,
}

impl Config {
  pub(crate) fn capture(&self, extension: &str) -> Utf8PathBuf {
    let filename = format!(
      "{}.{extension}",
      SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs(),
    );

    if let Some(captures) = &self.captures {
      captures.join(filename)
    } else {
      filename.into()
    }
  }

  pub(crate) fn find_song(&self, song: &str) -> Result<Utf8PathBuf> {
    let song = RegexBuilder::new(song)
      .case_insensitive(true)
      .build()
      .context(error::SongRegex)?;

    let mut matches = Vec::<Utf8PathBuf>::new();

    let music = self.music()?;

    for entry in WalkDir::new(music) {
      let entry = entry.context(error::SongWalk)?;

      if entry.file_type().is_dir() {
        continue;
      }

      let path = entry.path();

      let haystack = path.strip_prefix(music).unwrap().with_extension("");

      let Some(haystack) = haystack.to_str() else {
        continue;
      };

      if song.is_match(haystack) {
        matches.push(path.into_utf8_path()?.into());
      }
    }

    if matches.len() > 1 {
      return Err(error::SongAmbiguous { matches }.build());
    }

    match matches.into_iter().next() {
      Some(path) => Ok(path),
      None => Err(error::SongMatch { song }.build()),
    }
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
