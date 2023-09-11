use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use anyhow::Result;
use clap::Parser;
use log::{debug, info};
use notify::{Event, PollWatcher, RecommendedWatcher, RecursiveMode, Watcher};
use yaml_rust::Yaml;

/// Parses a file and returns [Ok(Path)] if the file should be synced
fn parse_front_matter<P: AsRef<std::path::Path>>(path: P) -> Result<()> {
    // TODO: learn how the question mark works here?
    let md = markdown_parser::read_file(path)?;

    // Validate front matter is expected format
    match md.format() {
        markdown_parser::Format::YAML => (),
        _ => anyhow::bail!("Unexpected macro input"),
    }

    // Parse yaml
    let yaml = yaml_rust::YamlLoader::load_from_str(md.front_matter())?;

    // TODO: more graceful handling
    match yaml[0]["sync"] {
        Yaml::Boolean(sync_bool) => match sync_bool {
            true => Ok(()),
            false => anyhow::bail!("No sync"),
        },
        Yaml::BadValue => anyhow::bail!("`sync` not present in front matter!"),
        _ => anyhow::bail!("`sync` is unexpected yaml type!"),
    }
}

// What are the common operations that I'd like to support
// How do I write tests to cover these use cases?

fn process_event(event: &Event, _personal: &PathBuf, _work: &PathBuf) -> Result<()> {
    info!("{:?}", event);
    // match event.kind{
    //     notify::EventKind::Access(notify::event::AccessKind::Close(_)) => {
    //         // File was closed, check for sync
    //         assert_eq!(event.paths.len(), 1);
    //         let path = &event.paths[0];
    //         let Ok(()) = parse_front_matter(path) else {
    //             return Ok(());
    //         };

    //         // If we're here it means that this file should be synced
    //         debug!("Sync candidate {}", path.display());

    //         // What checks do I want to do here?
    //         // Find target in other vault
    //         // Need to determine which vault changed
    //         // if path.starts_with(personal) {
    //         //     // file updated in personal vault, time to sync it to the work vault
    //         //     // determine subset of path
    //         //     let no_prefix = path.strip_prefix(personal)?;
    //         //     let to_path = work.clone().join(no_prefix);
    //         //     std::fs::copy(path, to_path)?;
    //         // }
    //         // Ok(())
    //         Ok(())
    //     },
    //     notify::EventKind::Access(_) => todo!(),
    //     notify::EventKind::Create(_) => todo!(),
    //     notify::EventKind::Modify(_) => todo!(),
    //     notify::EventKind::Remove(_) => todo!(),
    //     notify::EventKind::Any => todo!(),
    //     notify::EventKind::Other => todo!(),
    // }
    Ok(())
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about=None)]
struct Cli {
    /// Path to the first vault to watch
    vault_one: PathBuf,
    /// Path to the second vault to watch
    vault_two: PathBuf,
}

fn watch_directories(dir1: &Path) {
    let (tx, rx) = std::sync::mpsc::channel();
    let config = notify::Config::default().with_poll_interval(Duration::from_secs(1));
    let mut watcher = PollWatcher::new(tx, config).unwrap();
    // let mut watcher = RecommendedWatcher::new(tx, notify::Config::default()).unwrap();

    println!("Watching directories: {} and {}", dir1.display(), "dir2");

    // Watching a vault
    watcher.watch(&dir1, RecursiveMode::Recursive).unwrap();
    // watcher.watch(&dir2, RecursiveMode::Recursive).unwrap();

    // Wait for events
    for res in rx {
        match res {
            Ok(_event) => {
                println!("Hit an event!!!");
                return;
            }
            Err(error) => println!("Error: {error:?}"),
        }
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    println!("{:#?}", cli);

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = RecommendedWatcher::new(tx, notify::Config::default())?;

    // Watching a vault
    watcher.watch(&cli.vault_one, RecursiveMode::Recursive)?;
    watcher.watch(&cli.vault_two, RecursiveMode::Recursive)?;

    // Wait for events
    for res in rx {
        match res {
            Ok(event) => process_event(&event, &cli.vault_one, &cli.vault_two)?,
            Err(error) => log::error!("Error: {error:?}"),
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {

    use std::fs::File;

    use tempfile::tempdir;

    use crate::watch_directories;

    #[test]
    fn file_created() -> Result<(), std::io::Error> {
        let tmp_dir = tempdir()?;
        let tmp_dir_path = tmp_dir.path();
        let thread_1 = std::thread::spawn(move || {
            watch_directories(tmp_dir_path.clone());
        });

        let file_path = tmp_dir.into_path().join("my-temporary-note.txt");
        let mut _tmp_file = File::create(file_path)?;
        // Explicitly drop the file to hopefully trigger an event
        drop(_tmp_file);

        thread_1.join().unwrap();

        Ok(())
    }
}
