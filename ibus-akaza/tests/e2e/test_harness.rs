use anyhow::{Context, Result};
use std::process::{Child, Command};
use std::thread;
use std::time::Duration;

/// IBus daemon と ibus-akaza エンジンをセットアップするテストハーネス
pub struct IBusTestHarness {
    ibus_daemon: Option<Child>,
    engine_process: Option<Child>,
}

impl IBusTestHarness {
    /// テスト環境をセットアップ
    /// 1. ibus-daemon を起動
    /// 2. ibus-akaza エンジンを登録
    /// 3. akaza エンジンを有効化
    pub fn setup() -> Result<Self> {
        // 既存の ibus-daemon を終了
        let _ = Command::new("pkill").arg("-f").arg("ibus-daemon").status();
        thread::sleep(Duration::from_secs(1));

        // 1. ibus-daemon を起動
        let ibus_daemon = Command::new("ibus-daemon")
            .args(["--xim", "--daemonize", "--replace"])
            .spawn()
            .context("Failed to start ibus-daemon")?;

        thread::sleep(Duration::from_secs(3));

        // 2. ibus-akaza エンジンを登録
        let engine_process = Command::new("./target/debug/ibus-akaza")
            .arg("--ibus")
            .spawn()
            .context("Failed to start ibus-akaza engine")?;

        thread::sleep(Duration::from_secs(3));

        // 3. akaza エンジンを有効化
        let status = Command::new("ibus")
            .args(["engine", "akaza"])
            .status()
            .context("Failed to activate akaza engine")?;

        if !status.success() {
            anyhow::bail!("Failed to activate akaza engine");
        }

        thread::sleep(Duration::from_secs(2));

        Ok(IBusTestHarness {
            ibus_daemon: Some(ibus_daemon),
            engine_process: Some(engine_process),
        })
    }
}

impl Drop for IBusTestHarness {
    fn drop(&mut self) {
        // クリーンアップ: プロセスを終了
        if let Some(mut process) = self.engine_process.take() {
            let _ = process.kill();
        }
        if let Some(mut daemon) = self.ibus_daemon.take() {
            let _ = daemon.kill();
        }

        // 念のため pkill で確実に終了
        let _ = Command::new("pkill").arg("-f").arg("ibus-akaza").status();
        let _ = Command::new("pkill").arg("-f").arg("ibus-daemon").status();

        thread::sleep(Duration::from_millis(500));
    }
}

/// xdotool を使用してキーを送信
pub fn send_keys(text: &str) -> Result<()> {
    Command::new("xdotool")
        .args(["type", "--delay", "50", text])
        .status()
        .context("Failed to send keys with xdotool")?;
    Ok(())
}

/// xdotool を使用して特殊キーを送信
pub fn send_key(key_name: &str) -> Result<()> {
    Command::new("xdotool")
        .args(["key", key_name])
        .status()
        .context(format!("Failed to send key: {}", key_name))?;
    Ok(())
}

/// テスト用のアプリケーションウィンドウを開く
pub fn open_test_window() -> Result<Child> {
    let process = Command::new("xterm")
        .args(["-e", "cat"])
        .spawn()
        .context("Failed to open xterm")?;

    thread::sleep(Duration::from_secs(2));
    Ok(process)
}
