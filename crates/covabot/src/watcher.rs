use crate::personality::PersonalityStore;
use notify::event::ModifyKind;
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
        tokio::fs::write(&path_owned, "").await?;
    }

    watcher.watch(p, RecursiveMode::NonRecursive)?;

    tokio::spawn(async move {
        // Keep watcher alive
        let _watcher = watcher;
        while let Some(event) = rx.recv().await {
            match event.kind {
                EventKind::Modify(ModifyKind::Data(_)) | EventKind::Modify(ModifyKind::Any) => {
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
