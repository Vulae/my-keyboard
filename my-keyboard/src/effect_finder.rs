use anyhow::{anyhow, Result};
use notify::Watcher;
use rand::seq::IndexedRandom as _;
use std::path::{Path, PathBuf};

fn is_lua_file<P: AsRef<Path>>(path: P) -> bool {
    let path = path.as_ref();
    let Some(name) = path.file_name() else {
        return false;
    };
    let Some(name) = name.to_str() else {
        return false;
    };
    name.ends_with(".lua") && !name.ends_with(".d.lua")
}

#[derive(Debug)]
pub struct EffectFinder {
    path: PathBuf,
    is_target_file: bool,
    forced_next: Option<PathBuf>,
    current_effect_path: Option<PathBuf>,
    #[allow(unused)]
    watcher: Option<notify::RecommendedWatcher>,
    rx: Option<std::sync::mpsc::Receiver<Result<notify::event::Event, notify::Error>>>,
}

impl EffectFinder {
    pub fn new<P: AsRef<Path>>(path: P, watch: bool) -> Result<EffectFinder> {
        let is_target_file = path.as_ref().metadata()?.is_file();
        if watch {
            let dir = if !is_target_file {
                path.as_ref()
            } else {
                path.as_ref()
                    .parent()
                    .ok_or(anyhow!("Watch file has no parent"))?
            };
            let (tx, rx) = std::sync::mpsc::channel();
            let mut watcher = notify::recommended_watcher(tx)?;
            watcher.watch(dir, notify::RecursiveMode::NonRecursive)?;
            log::info!("Watching {:#?}", dir);
            Ok(EffectFinder {
                path: path.as_ref().to_path_buf(),
                is_target_file,
                forced_next: None,
                current_effect_path: None,
                watcher: Some(watcher),
                rx: Some(rx),
            })
        } else {
            Ok(EffectFinder {
                path: path.as_ref().to_path_buf(),
                is_target_file,
                forced_next: None,
                current_effect_path: None,
                watcher: None,
                rx: None,
            })
        }
    }

    fn next_effect_inner(&mut self) -> Result<PathBuf> {
        if self.is_target_file {
            if !is_lua_file(&self.path) {
                return Err(anyhow!("File is not lua file"));
            }
            Ok(self.path.clone())
        } else {
            let entries: Vec<PathBuf> = self
                .path
                .read_dir()?
                .map(|entry| {
                    let entry = entry?;
                    let path = entry.path();
                    let meta = path.metadata()?;
                    if !meta.is_file() {
                        return Ok(None);
                    }
                    Ok(is_lua_file(&path).then_some(path))
                })
                .collect::<Result<Vec<Option<PathBuf>>>>()?
                .into_iter()
                .flatten()
                .collect();

            if entries.is_empty() {
                return Err(anyhow!("Directory has no valid lua files"));
            }
            if entries.len() == 1 {
                return Ok(entries[0].clone());
            }

            let entries: Vec<PathBuf> = entries
                .into_iter()
                .filter(|path| Some(path) != self.current_effect_path.as_ref())
                .collect();

            Ok(entries.choose(&mut rand::rng()).unwrap().clone())
        }
    }

    pub fn next_effect(&mut self) -> Result<PathBuf> {
        let path = if let Some(forced_next) = self.forced_next.take() {
            return Ok(forced_next);
        } else {
            self.next_effect_inner()?
        };
        self.current_effect_path = Some(path.clone());
        Ok(path)
    }

    pub fn dir_changed(&mut self) -> Result<bool> {
        let Some(rx) = self.rx.as_ref() else {
            return Ok(false);
        };
        let mut change = false;
        for event in rx.try_iter() {
            let event = event?;
            if matches!(
                event.kind,
                notify::EventKind::Create(_)
                    | notify::EventKind::Modify(_)
                    | notify::EventKind::Remove(_)
            ) {
                for path in &event.paths {
                    if !path.try_exists()? {
                        continue;
                    }
                    let meta = path.metadata()?;
                    if meta.is_file() && is_lua_file(path) {
                        if !self.is_target_file
                            && !matches!(event.kind, notify::EventKind::Remove(_))
                        {
                            self.forced_next = Some(path.to_owned());
                        }
                        change = true;
                    }
                }
            }
        }
        Ok(change)
    }
}
