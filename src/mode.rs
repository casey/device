#[derive(Debug)]
pub(crate) enum Mode {
  Command(Vec<String>),
  Normal,
  Play,
}
