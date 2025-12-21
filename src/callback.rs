use super::*;

pub(crate) trait Callback {
  fn call(&mut self, state: &mut State, dt: Duration);

  fn clone_box(&self) -> Box<dyn Callback>;
}

impl<T> Callback for T
where
  T: FnMut(&mut State, Duration) + Clone + 'static,
{
  fn call(&mut self, state: &mut State, dt: Duration) {
    self(state, dt);
  }

  fn clone_box(&self) -> Box<dyn Callback> {
    Box::new(self.clone())
  }
}

impl Clone for Box<dyn Callback> {
  fn clone(&self) -> Self {
    self.clone_box()
  }
}
