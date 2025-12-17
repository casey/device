use super::*;

pub(crate) fn default<T: Default>() -> T {
  T::default()
}

pub(crate) fn display<'a, T: Display + 'a>(t: T) -> Box<dyn Display + 'a> {
  Box::new(t)
}

pub(crate) fn pad(i: usize, alignment: usize) -> usize {
  assert!(alignment.is_power_of_two());
  (i + alignment - 1) & !(alignment - 1)
}

pub(crate) fn tempdir() -> Result<(TempDir, Utf8PathBuf)> {
  let tempdir = tempfile::Builder::new()
    .prefix("device")
    .tempdir()
    .context(error::TempdirIo)?;

  let path = tempdir.path().into_utf8_path()?.into();

  Ok((tempdir, path))
}

pub(crate) fn thread_spawn<F, T>(name: &str, f: F) -> Result<JoinHandle<T>>
where
  F: FnOnce() -> T + Send + 'static,
  T: Send + 'static,
{
  std::thread::Builder::new()
    .name(name.into())
    .spawn(f)
    .context(error::ThreadSpawn { name })
}
