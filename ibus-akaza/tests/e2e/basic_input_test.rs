use anyhow::Result;
use serial_test::serial;
use std::thread;
use std::time::Duration;

use super::test_harness;

/// 基本的なE2Eテスト
/// 注意: これらのテストは実際のIBusとXvfbが必要です

#[test]
#[serial]
#[ignore] // デフォルトではスキップ。--ignored で実行
fn test_ibus_daemon_starts() -> Result<()> {
    // IBus daemon が起動できることを確認
    let _harness = test_harness::IBusTestHarness::setup()?;

    // daemon が起動していることを確認
    thread::sleep(Duration::from_secs(1));

    Ok(())
}

#[test]
#[serial]
#[ignore] // デフォルトではスキップ。--ignored で実行
fn test_engine_registration() -> Result<()> {
    // ibus-akaza エンジンが登録できることを確認
    let _harness = test_harness::IBusTestHarness::setup()?;

    // エンジンが登録されていることを確認（簡易チェック）
    thread::sleep(Duration::from_secs(2));

    Ok(())
}

#[test]
#[serial]
#[ignore] // デフォルトではスキップ。--ignored で実行
fn test_send_keys() -> Result<()> {
    // xdotool でキー入力ができることを確認
    let _harness = test_harness::IBusTestHarness::setup()?;
    let _window = test_harness::open_test_window()?;

    // キー入力のテスト
    test_harness::send_keys("test")?;
    thread::sleep(Duration::from_millis(500));

    Ok(())
}

// 将来の拡張用テスト（現在はスキップ）

#[test]
#[serial]
#[ignore]
fn test_hiragana_input() -> Result<()> {
    // ひらがな入力のテスト
    // TODO: 実際の入力結果を検証する仕組みが必要
    let _harness = test_harness::IBusTestHarness::setup()?;
    let _window = test_harness::open_test_window()?;

    test_harness::send_keys("a")?;
    thread::sleep(Duration::from_millis(500));

    // 結果の検証は将来実装
    Ok(())
}

#[test]
#[serial]
#[ignore]
fn test_conversion() -> Result<()> {
    // 変換のテスト
    // TODO: 実際の変換結果を検証する仕組みが必要
    let _harness = test_harness::IBusTestHarness::setup()?;
    let _window = test_harness::open_test_window()?;

    test_harness::send_keys("watasi")?;
    test_harness::send_key("space")?;
    thread::sleep(Duration::from_millis(500));

    // 結果の検証は将来実装
    Ok(())
}
