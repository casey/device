use super::*;

pub(crate) struct Codepoint(pub(crate) char);

impl Display for Codepoint {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "U+{:04X}", u32::from(self.0))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn codepoint() {
    assert_eq!(Codepoint('\0').to_string(), "U+0000");
  }
}
