pub(crate) trait IntoU128 {
  fn into_u128(self) -> u128;
}

impl IntoU128 for usize {
  fn into_u128(self) -> u128 {
    self.try_into().unwrap()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn usize_into_u128() {
    usize::MAX.into_u128();
  }
}
