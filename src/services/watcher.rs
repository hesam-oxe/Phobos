//! Phobos IDE - File Watcher (notify)
use std::path::PathBuf;
use std::sync::mpsc as std_mpsc;
use notify::{Watcher, RecursiveMode, RecommendedWatcher, Config};

pub enum WatchEvent { Changed(PathBuf), Created(PathBuf), Removed(PathBuf) }

pub fn create_watcher() -> Result<(RecommendedWatcher, std_mpsc::Receiver<WatchEvent>), notify::Error> {
    let (tx, rx) = std_mpsc::channel();
    let watcher = RecommendedWatcher::new(
        move |res: notify::Result<notify::Event>| {
            if let Ok(ev) = res {
                for p in ev.paths {
                    let e = match ev.kind {
                        notify::EventKind::Create(_) => WatchEvent::Created(p),
                        notify::EventKind::Remove(_) => WatchEvent::Removed(p),
                        _ => WatchEvent::Changed(p),
                    };
                    let _ = tx.send(e);
                }
            }
        },
        Config::default(),
    )?;
    Ok((watcher, rx))
}

pub fn watch_path(watcher: &mut RecommendedWatcher, path: &str) -> bool {
    watcher.watch(std::path::Path::new(path), RecursiveMode::Recursive).is_ok()
}
