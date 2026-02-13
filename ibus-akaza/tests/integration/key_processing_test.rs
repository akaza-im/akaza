use ibus_akaza_lib::commands::ibus_akaza_commands_map;
use ibus_akaza_lib::input_mode::{get_all_input_modes, get_input_mode_from_prop_name};
use ibus_akaza_lib::test_utils::mock_engine;

/// キー処理の基本的な統合テスト
/// IBusEngineの実際の機能は使わず、モジュール間の連携をテスト

#[test]
fn test_commands_can_be_retrieved() {
    // コマンドマップが正しく取得できることを確認
    let commands = ibus_akaza_commands_map();

    // 基本的なコマンドが存在することを確認
    assert!(commands.contains_key("commit_candidate"));
    assert!(commands.contains_key("escape"));
    assert!(commands.contains_key("update_candidates"));
}

#[test]
fn test_input_modes_integration() {
    // 入力モードの取得と変換が正しく動作することを確認
    let modes = get_all_input_modes();
    assert_eq!(modes.len(), 5);

    // 各モードがprop_nameで取得できることを確認
    for mode in modes {
        let retrieved = get_input_mode_from_prop_name(mode.prop_name).unwrap();
        assert_eq!(retrieved, *mode);
    }
}

#[test]
fn test_mock_engine_is_null() {
    // mock_engineがnullポインタを返すことを確認
    let engine = mock_engine();
    assert!(engine.is_null(), "mock_engine should return null pointer");
}

#[test]
fn test_command_names_are_consistent() {
    // コマンド名が一貫した命名規則に従っていることを確認
    let commands = ibus_akaza_commands_map();

    // 入力モード設定コマンドが全て set_input_mode_ で始まることを確認
    let mode_commands: Vec<_> = commands
        .keys()
        .filter(|&k| k.starts_with("set_input_mode_"))
        .collect();

    assert!(
        mode_commands.len() >= 5,
        "Should have at least 5 input mode commands"
    );

    // 数字コマンドが全て press_number_ で始まることを確認
    let number_commands: Vec<_> = commands
        .keys()
        .filter(|&k| k.starts_with("press_number_"))
        .collect();

    assert_eq!(
        number_commands.len(),
        10,
        "Should have exactly 10 number commands (0-9)"
    );
}

#[test]
fn test_conversion_commands_exist() {
    // 変換コマンドが全て存在することを確認
    let commands = ibus_akaza_commands_map();

    let conversion_types = vec![
        "convert_to_full_hiragana",
        "convert_to_full_katakana",
        "convert_to_half_katakana",
        "convert_to_full_romaji",
        "convert_to_half_romaji",
    ];

    for cmd in conversion_types {
        assert!(
            commands.contains_key(cmd),
            "Missing conversion command: {}",
            cmd
        );
    }
}
