#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Ord, PartialOrd, strum::Display)]
#[strum(serialize_all = "kebab-case")]
pub(crate) enum Press {
  Press,
  Release,
}
