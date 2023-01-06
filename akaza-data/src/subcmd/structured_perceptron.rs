use std::collections::HashMap;

use libakaza::akaza_builder::AkazaBuilder;

/// 構造化パーセプトロンの学習を行います。
/// 構造化パーセプトロンは、シンプルな実装で、そこそこのパフォーマンスがでる(予定)
/// 構造化パーセプトロンでいい感じに動くようならば、構造化SVMなどに挑戦したい。
pub fn learn_structured_perceptron() -> anyhow::Result<()> {
    let akaza = AkazaBuilder::default().build()?;

    let force = Vec::new();
    let lattice = akaza.to_lattice("ほげほげ", &force)?;
    let result = akaza.resolve(&lattice)?;

    Ok(())
}
