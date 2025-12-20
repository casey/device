use super::*;

pub(crate) struct Interrupt {
  interrupted: Arc<AtomicBool>,
}

impl Interrupt {
  pub(crate) fn interrupted(&self) -> bool {
    self.interrupted.load(atomic::Ordering::Relaxed)
  }

  pub(crate) fn register() -> Result<Self> {
    let interrupted = Arc::new(AtomicBool::new(false));

    signal_hook::flag::register(libc::SIGINT, interrupted.clone())
      .context(error::SignalRegister)?;

    Ok(Self { interrupted })
  }
}
