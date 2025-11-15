use {
  super::*,
  blake3::{Hash, Hasher},
  notify::{Event, EventHandler, RecommendedWatcher, RecursiveMode, Watcher},
};

pub(crate) struct Binary {
  watcher: RecommendedWatcher,
}

impl Binary {
  pub(crate) fn new(path: &Utf8Path) -> Result<Self> {
    let path = env::current_dir()
      .context(error::CurrentDir)?
      .into_utf8_path()?
      .join(path);

    let name = path.file_name().unwrap().to_owned();

    let parent = path.parent().unwrap().to_owned();

    eprintln!("watching {parent} for {name}");

    let handler = Handler { path, hash: None };

    let mut watcher = RecommendedWatcher::new(handler, notify::Config::default()).unwrap();

    watcher
      .watch(parent.as_ref(), RecursiveMode::NonRecursive)
      .unwrap();

    Ok(Self { watcher })
  }
}

struct Handler {
  hash: Option<Hash>,
  path: Utf8PathBuf,
}

impl EventHandler for Handler {
  fn handle_event(&mut self, event: notify::Result<Event>) {
    let event = match event {
      Err(err) => {
        log::error!("watch error: {err}");
        return;
      }
      Ok(event) => event,
    };

    if event.paths.iter().all(|path| *path != self.path) {
      log::info!("ignoring event");
      return;
    }

    if let Some(old) = self.hash {
      let mut hasher = Hasher::new();

      if let Err(err) = hasher.update_mmap_rayon(&self.path) {
        log::error!("failed to hash binary: {err}");
        return;
      }

      let new = hasher.finalize();

      if new == old {
        log::info!("binary did not change");
        return;
      }
    }

    let child = Command::new(&self.path)
      .stdout(Stdio::piped())
      .spawn()
      .unwrap();

    let mut reader = BufReader::new(child.stdout.unwrap());
    loop {
      let mut line = String::new();
      reader.read_line(&mut line).unwrap();
      eprint!("{line}");
    }
  }
}
