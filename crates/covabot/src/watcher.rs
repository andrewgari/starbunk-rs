use crate::personality::PersonalityStore;
use notify::event::{ModifyKind, RenameMode};
use notify::{EventKind, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::Arc;

pub async fn watch_personality_file(
    store: Arc<PersonalityStore>,
    path: &str,
) -> anyhow::Result<()> {
    let path_owned = path.to_string();

    // Create an async channel to receive events
    let (tx, mut rx) = tokio::sync::mpsc::channel(100);

    let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
        if let Ok(event) = res {
            let _ = tx.blocking_send(event);
        }
    })?;

    let p = Path::new(&path_owned);
    if let Some(parent) = p.parent() {
        if !parent.exists() {
            tokio::fs::create_dir_all(parent).await?;
        }
    }
    if !p.exists() {
        if let Err(e) = tokio::fs::write(&path_owned, "").await {
            tracing::warn!(path = %path_owned, err = %e, "Could not create personality file; watcher may not work if the file does not exist");
        }
    }

    if let Err(e) = watcher.watch(p, RecursiveMode::NonRecursive) {
        tracing::warn!("Failed to watch {}: {}", path_owned, e);
        // Try watching the parent directory as a fallback for kubernetes configmaps
        if let Some(parent) = p.parent() {
            if let Err(e2) = watcher.watch(parent, RecursiveMode::NonRecursive) {
                tracing::error!("Failed to watch parent directory either: {}", e2);
                return Err(e2.into());
            } else {
                tracing::info!(
                    "Watching parent directory {} instead (Kubernetes fallback)",
                    parent.display()
                );
            }
        } else {
            return Err(e.into());
        }
    }

    tokio::spawn(async move {
        // Keep watcher alive
        let _watcher = watcher;
        while let Some(event) = rx.recv().await {
            let target_path_matches = event.paths.iter().any(|p| p.ends_with(&path_owned));
            match event.kind {
                EventKind::Modify(ModifyKind::Data(_))
                | EventKind::Modify(ModifyKind::Any)
                | EventKind::Create(_)
                | EventKind::Modify(ModifyKind::Name(RenameMode::To))
                    if target_path_matches || event.paths.is_empty() =>
                {
                    tracing::info!("Detected modification in {}", path_owned);
                    if let Ok(content) = tokio::fs::read_to_string(&path_owned).await {
                        if let Err(e) = store.sync_from_yaml(&content).await {
                            tracing::error!("Failed to sync from yaml: {}", e);
                        } else {
                            tracing::info!("Successfully synced personality from YAML file to DB.");
                        }
                    }
                }
                _ => {}
            }
        }
    });

    Ok(())
}
