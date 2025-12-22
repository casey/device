use {super::*, std::process::Command};

pub(crate) trait CommandExt {
  fn new_process_group(&mut self) -> &mut Self;

  fn stdout_utf8(&mut self) -> Result<String>;
}

impl CommandExt for Command {
  fn new_process_group(&mut self) -> &mut Self {
    use std::os::unix::process::CommandExt;

    unsafe {
      self.pre_exec(|| {
        if libc::setpgid(0, 0) != 0 {
          return Err(std::io::Error::last_os_error());
        }

        Ok(())
      });
    }

    self
  }

  fn stdout_utf8(&mut self) -> Result<String> {
    let output = self.output().context(error::CommandRun {
      program: self.get_program(),
    })?;

    ensure! {
      output.status.success(),
      error::CommandStatus {
        program: self.get_program(),
        status: output.status,
        stderr: output.stderr,
      },
    }

    String::from_utf8(output.stdout).context(error::CommandUtf8 {
      program: self.get_program(),
    })
  }
}
