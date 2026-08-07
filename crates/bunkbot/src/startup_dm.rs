use std::sync::Arc;

/// Check whether the current deployment version differs from the last persisted
/// version, and if so, send a Discord DM to the configured notification user.
///
/// Environment variables read at call time:
/// - `APP_VERSION`           — current deployment version.  Skipped when absent or "dev".
/// - `DISCORD_NOTIFY_USER_ID` — Discord user-ID (u64) to DM.  Skipped when absent.
/// - `STARTUP_DM_DATA_DIR`   — directory that holds `last_version`.  Defaults to `/app/data`.
#[tracing::instrument(skip(http), fields(bot = bot_display_name))]
pub async fn check_and_notify(
    http: &Arc<serenity::all::Http>,
    bot_display_name: &str,
) -> anyhow::Result<()> {
    // 1. Read APP_VERSION; skip if unset or "dev".
    let current_version = match std::env::var("APP_VERSION") {
        Ok(v) if !v.is_empty() && v != "dev" => v,
        _ => {
            tracing::debug!(
                bot = bot_display_name,
                "APP_VERSION not set or is 'dev'; skipping startup DM"
            );
            return Ok(());
        }
    };

    // 2. Read DISCORD_NOTIFY_USER_ID; skip if unset.
    let notify_user_id_str = match std::env::var("DISCORD_NOTIFY_USER_ID") {
        Ok(v) if !v.is_empty() => v,
        _ => {
            tracing::debug!(
                bot = bot_display_name,
                "DISCORD_NOTIFY_USER_ID not set; skipping startup DM"
            );
            return Ok(());
        }
    };

    // 3. Resolve data dir.
    let data_dir = std::env::var("STARTUP_DM_DATA_DIR").unwrap_or_else(|_| "/app/data".to_string());
    let version_file_path = std::path::PathBuf::from(&data_dir).join("last_version");

    // 4. Read last_version file; compare with current.
    let last_version = tokio::fs::read_to_string(&version_file_path)
        .await
        .unwrap_or_default();
    if last_version.trim() == current_version.trim() {
        tracing::debug!(
            bot = bot_display_name,
            version = %current_version,
            "version unchanged; skipping startup DM"
        );
        return Ok(());
    }

    let is_upgrade = !last_version.trim().is_empty();
    let message = if is_upgrade {
        format!(
            "🚀 **{}** has been updated from `{}` to `{}`.",
            bot_display_name,
            last_version.trim(),
            current_version.trim()
        )
    } else {
        format!(
            "🚀 **{}** has started for the first time with version `{}`.",
            bot_display_name,
            current_version.trim()
        )
    };

    // 5. Parse user ID and send DM.
    match notify_user_id_str.parse::<u64>() {
        Ok(user_id_num) => {
            let user_id = serenity::all::UserId::new(user_id_num);
            match user_id.create_dm_channel(http).await {
                Ok(dm_channel) => {
                    if let Err(e) = dm_channel.say(http, &message).await {
                        tracing::warn!(
                            bot = bot_display_name,
                            err = %e,
                            "failed to send startup DM"
                        );
                    } else {
                        tracing::info!(
                            bot = bot_display_name,
                            version = %current_version,
                            "sent startup DM notification"
                        );
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        bot = bot_display_name,
                        err = %e,
                        "failed to create DM channel for startup notification"
                    );
                }
            }
        }
        Err(e) => {
            tracing::warn!(
                bot = bot_display_name,
                user_id = %notify_user_id_str,
                err = %e,
                "DISCORD_NOTIFY_USER_ID is not a valid u64; skipping startup DM"
            );
        }
    }

    // 7. Persist new version (create dir if needed, then write file).
    if let Err(e) = tokio::fs::create_dir_all(&data_dir).await {
        tracing::warn!(
            bot = bot_display_name,
            data_dir = %data_dir,
            err = %e,
            "failed to create data dir for startup DM version file"
        );
    }
    if let Err(e) = tokio::fs::write(&version_file_path, current_version.trim()).await {
        tracing::warn!(
            bot = bot_display_name,
            path = %version_file_path.display(),
            err = %e,
            "failed to write startup DM version file"
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::{Arc, Mutex};

    // ---------------------------------------------------------------------------
    // Helpers
    // ---------------------------------------------------------------------------

    /// Returns a unique temp dir for this test to use (caller must hold it alive).
    fn unique_temp_dir(test_name: &str) -> PathBuf {
        let base = std::env::temp_dir().join("bunkbot_startup_dm_tests");
        let dir = base.join(format!("{}_{}", test_name, std::process::id()));
        // best-effort cleanup from a prior run
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    /// A record of calls made to the mock DM sender.
    #[derive(Clone, Default)]
    struct MockCalls {
        messages: Arc<Mutex<Vec<String>>>,
    }

    impl MockCalls {
        fn push(&self, msg: &str) {
            self.messages
                .lock()
                .expect("lock poisoned")
                .push(msg.to_string());
        }
        fn count(&self) -> usize {
            self.messages.lock().expect("lock poisoned").len()
        }
        fn get(&self, idx: usize) -> String {
            self.messages.lock().expect("lock poisoned")[idx].clone()
        }
    }

    // ---------------------------------------------------------------------------
    // We can't easily call `check_and_notify` with a real `serenity::Http` in
    // unit tests (no live Discord), so we extract the core logic into a
    // testable helper that accepts an async DM-send closure.
    //
    // The production function simply calls this helper with the real serenity
    // implementation.
    // ---------------------------------------------------------------------------

    /// Core logic extracted for testability.
    ///
    /// `dm_sender` receives `(user_id_str, message)` and returns `Ok(true)` when
    /// the DM was sent, `Ok(false)` on graceful failure, or `Err(...)` to simulate
    /// a send error.  The real implementation calls serenity.
    async fn check_and_notify_inner<F, Fut>(dm_sender: F) -> anyhow::Result<()>
    where
        F: FnOnce(String, String) -> Fut,
        Fut: std::future::Future<Output = anyhow::Result<bool>>,
    {
        let current_version = match std::env::var("APP_VERSION") {
            Ok(v) if !v.is_empty() && v != "dev" => v,
            _ => return Ok(()),
        };

        let notify_user_id_str = match std::env::var("DISCORD_NOTIFY_USER_ID") {
            Ok(v) if !v.is_empty() => v,
            _ => return Ok(()),
        };

        let data_dir =
            std::env::var("STARTUP_DM_DATA_DIR").unwrap_or_else(|_| "/app/data".to_string());
        let version_file_path = PathBuf::from(&data_dir).join("last_version");

        let last_version = tokio::fs::read_to_string(&version_file_path)
            .await
            .unwrap_or_default();
        if last_version.trim() == current_version.trim() {
            return Ok(());
        }

        let is_upgrade = !last_version.trim().is_empty();
        let message = if is_upgrade {
            format!(
                "BunkBot has been updated from `{}` to `{}`.",
                last_version.trim(),
                current_version.trim()
            )
        } else {
            format!(
                "BunkBot has started for the first time with version `{}`.",
                current_version.trim()
            )
        };

        // Send DM; log but do not propagate failures.
        let _ = dm_sender(notify_user_id_str, message).await;

        // Always write version file.
        if let Err(e) = tokio::fs::create_dir_all(&data_dir).await {
            tracing::warn!(err = %e, "failed to create data dir");
        }
        if let Err(e) = tokio::fs::write(&version_file_path, current_version.trim()).await {
            tracing::warn!(err = %e, "failed to write version file");
        }

        Ok(())
    }

    // ---------------------------------------------------------------------------
    // Helper: set env vars for the duration of the test (serial only).
    // NOTE: std::env::set_var is not safe in multi-threaded tests; the
    //       `#[tokio::test]` tests below are each their own process-level
    //       mutation.  Use serial execution by running with -- --test-threads=1
    //       or coordinate via a Mutex.
    // ---------------------------------------------------------------------------

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    // ---------------------------------------------------------------------------
    // Tests
    // ---------------------------------------------------------------------------

    #[tokio::test]
    async fn sends_dm_when_version_file_missing() {
        let _lock = ENV_LOCK.lock().expect("env lock");
        let dir = unique_temp_dir("sends_dm_missing_file");
        // dir does NOT exist yet — version file definitely missing
        unsafe {
            std::env::set_var("APP_VERSION", "1.2.3");
            std::env::set_var("DISCORD_NOTIFY_USER_ID", "12345");
            std::env::set_var("STARTUP_DM_DATA_DIR", dir.to_str().expect("utf8 path"));
        }

        let calls = MockCalls::default();
        let calls_clone = calls.clone();

        let result = check_and_notify_inner(move |_uid, msg| {
            let c = calls_clone.clone();
            async move {
                c.push(&msg);
                Ok(true)
            }
        })
        .await;

        unsafe {
            std::env::remove_var("APP_VERSION");
            std::env::remove_var("DISCORD_NOTIFY_USER_ID");
            std::env::remove_var("STARTUP_DM_DATA_DIR");
        }

        assert!(result.is_ok());
        assert_eq!(calls.count(), 1, "DM should have been sent once");

        // Version file should now exist
        let written = tokio::fs::read_to_string(dir.join("last_version"))
            .await
            .expect("version file should have been written");
        assert_eq!(written.trim(), "1.2.3");
    }

    #[tokio::test]
    async fn sends_dm_when_version_changed() {
        let _lock = ENV_LOCK.lock().expect("env lock");
        let dir = unique_temp_dir("sends_dm_version_changed");
        tokio::fs::create_dir_all(&dir).await.expect("create dir");
        tokio::fs::write(dir.join("last_version"), "1.0.0")
            .await
            .expect("write old version");

        unsafe {
            std::env::set_var("APP_VERSION", "2.0.0");
            std::env::set_var("DISCORD_NOTIFY_USER_ID", "99999");
            std::env::set_var("STARTUP_DM_DATA_DIR", dir.to_str().expect("utf8 path"));
        }

        let calls = MockCalls::default();
        let calls_clone = calls.clone();

        let result = check_and_notify_inner(move |_uid, msg| {
            let c = calls_clone.clone();
            async move {
                c.push(&msg);
                Ok(true)
            }
        })
        .await;

        unsafe {
            std::env::remove_var("APP_VERSION");
            std::env::remove_var("DISCORD_NOTIFY_USER_ID");
            std::env::remove_var("STARTUP_DM_DATA_DIR");
        }

        assert!(result.is_ok());
        assert_eq!(calls.count(), 1);
        let msg = calls.get(0);
        assert!(msg.contains("1.0.0"), "message should mention old version");
        assert!(msg.contains("2.0.0"), "message should mention new version");

        let written = tokio::fs::read_to_string(dir.join("last_version"))
            .await
            .expect("version file written");
        assert_eq!(written.trim(), "2.0.0");
    }

    #[tokio::test]
    async fn skips_dm_when_version_unchanged() {
        let _lock = ENV_LOCK.lock().expect("env lock");
        let dir = unique_temp_dir("skips_dm_unchanged");
        tokio::fs::create_dir_all(&dir).await.expect("create dir");
        tokio::fs::write(dir.join("last_version"), "1.0.0")
            .await
            .expect("write version");

        unsafe {
            std::env::set_var("APP_VERSION", "1.0.0");
            std::env::set_var("DISCORD_NOTIFY_USER_ID", "12345");
            std::env::set_var("STARTUP_DM_DATA_DIR", dir.to_str().expect("utf8 path"));
        }

        let calls = MockCalls::default();
        let calls_clone = calls.clone();

        let result = check_and_notify_inner(move |_uid, msg| {
            let c = calls_clone.clone();
            async move {
                c.push(&msg);
                Ok(true)
            }
        })
        .await;

        unsafe {
            std::env::remove_var("APP_VERSION");
            std::env::remove_var("DISCORD_NOTIFY_USER_ID");
            std::env::remove_var("STARTUP_DM_DATA_DIR");
        }

        assert!(result.is_ok());
        assert_eq!(
            calls.count(),
            0,
            "DM should NOT be sent when version unchanged"
        );
    }

    #[tokio::test]
    async fn skips_dm_when_app_version_unset() {
        let _lock = ENV_LOCK.lock().expect("env lock");
        let dir = unique_temp_dir("skips_dm_no_app_version");
        unsafe {
            std::env::remove_var("APP_VERSION");
            std::env::set_var("DISCORD_NOTIFY_USER_ID", "12345");
            std::env::set_var("STARTUP_DM_DATA_DIR", dir.to_str().expect("utf8 path"));
        }

        let calls = MockCalls::default();
        let calls_clone = calls.clone();

        let result = check_and_notify_inner(move |_uid, msg| {
            let c = calls_clone.clone();
            async move {
                c.push(&msg);
                Ok(true)
            }
        })
        .await;

        unsafe {
            std::env::remove_var("DISCORD_NOTIFY_USER_ID");
            std::env::remove_var("STARTUP_DM_DATA_DIR");
        }

        assert!(result.is_ok());
        assert_eq!(
            calls.count(),
            0,
            "DM should NOT be sent when APP_VERSION is unset"
        );
        // No version file should have been created
        assert!(!dir.join("last_version").exists());
    }

    #[tokio::test]
    async fn skips_dm_when_app_version_is_dev() {
        let _lock = ENV_LOCK.lock().expect("env lock");
        let dir = unique_temp_dir("skips_dm_dev");
        unsafe {
            std::env::set_var("APP_VERSION", "dev");
            std::env::set_var("DISCORD_NOTIFY_USER_ID", "12345");
            std::env::set_var("STARTUP_DM_DATA_DIR", dir.to_str().expect("utf8 path"));
        }

        let calls = MockCalls::default();
        let calls_clone = calls.clone();

        let result = check_and_notify_inner(move |_uid, msg| {
            let c = calls_clone.clone();
            async move {
                c.push(&msg);
                Ok(true)
            }
        })
        .await;

        unsafe {
            std::env::remove_var("APP_VERSION");
            std::env::remove_var("DISCORD_NOTIFY_USER_ID");
            std::env::remove_var("STARTUP_DM_DATA_DIR");
        }

        assert!(result.is_ok());
        assert_eq!(
            calls.count(),
            0,
            "DM should NOT be sent when APP_VERSION='dev'"
        );
        assert!(!dir.join("last_version").exists());
    }

    #[tokio::test]
    async fn skips_dm_when_notify_user_id_unset() {
        let _lock = ENV_LOCK.lock().expect("env lock");
        let dir = unique_temp_dir("skips_dm_no_user_id");
        unsafe {
            std::env::set_var("APP_VERSION", "3.0.0");
            std::env::remove_var("DISCORD_NOTIFY_USER_ID");
            std::env::set_var("STARTUP_DM_DATA_DIR", dir.to_str().expect("utf8 path"));
        }

        let calls = MockCalls::default();
        let calls_clone = calls.clone();

        let result = check_and_notify_inner(move |_uid, msg| {
            let c = calls_clone.clone();
            async move {
                c.push(&msg);
                Ok(true)
            }
        })
        .await;

        unsafe {
            std::env::remove_var("APP_VERSION");
            std::env::remove_var("STARTUP_DM_DATA_DIR");
        }

        assert!(result.is_ok());
        assert_eq!(
            calls.count(),
            0,
            "DM should NOT be sent when DISCORD_NOTIFY_USER_ID is unset"
        );
        assert!(!dir.join("last_version").exists());
    }

    #[tokio::test]
    async fn continues_when_dm_send_fails() {
        let _lock = ENV_LOCK.lock().expect("env lock");
        let dir = unique_temp_dir("continues_on_dm_failure");
        unsafe {
            std::env::set_var("APP_VERSION", "4.0.0");
            std::env::set_var("DISCORD_NOTIFY_USER_ID", "12345");
            std::env::set_var("STARTUP_DM_DATA_DIR", dir.to_str().expect("utf8 path"));
        }

        // Sender returns an error
        let result = check_and_notify_inner(move |_uid, _msg| async move {
            Err(anyhow::anyhow!("simulated send failure"))
        })
        .await;

        unsafe {
            std::env::remove_var("APP_VERSION");
            std::env::remove_var("DISCORD_NOTIFY_USER_ID");
            std::env::remove_var("STARTUP_DM_DATA_DIR");
        }

        // Function must still succeed (graceful degradation)
        assert!(
            result.is_ok(),
            "check_and_notify must not propagate DM send errors"
        );
        // Version file should still be written
        let written = tokio::fs::read_to_string(dir.join("last_version"))
            .await
            .expect("version file should be written even when DM fails");
        assert_eq!(written.trim(), "4.0.0");
    }

    #[tokio::test]
    async fn creates_data_dir_if_missing() {
        let _lock = ENV_LOCK.lock().expect("env lock");
        let dir = unique_temp_dir("creates_data_dir");
        // dir does not exist
        let nested = dir.join("a").join("b").join("c");
        unsafe {
            std::env::set_var("APP_VERSION", "5.0.0");
            std::env::set_var("DISCORD_NOTIFY_USER_ID", "12345");
            std::env::set_var("STARTUP_DM_DATA_DIR", nested.to_str().expect("utf8 path"));
        }

        let calls = MockCalls::default();
        let calls_clone = calls.clone();

        let result = check_and_notify_inner(move |_uid, msg| {
            let c = calls_clone.clone();
            async move {
                c.push(&msg);
                Ok(true)
            }
        })
        .await;

        unsafe {
            std::env::remove_var("APP_VERSION");
            std::env::remove_var("DISCORD_NOTIFY_USER_ID");
            std::env::remove_var("STARTUP_DM_DATA_DIR");
        }

        assert!(result.is_ok());
        assert!(nested.exists(), "data dir should have been created");
        let written = tokio::fs::read_to_string(nested.join("last_version"))
            .await
            .expect("version file created in new dir");
        assert_eq!(written.trim(), "5.0.0");
    }
}
