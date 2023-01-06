use clap::{Parser, Subcommand};

use crate::subcmd::make_system_dict::make_system_dict;
use crate::subcmd::make_system_lm::make_system_lm;

mod subcmd;

#[derive(Debug, Parser)]
#[clap(
name = env ! ("CARGO_PKG_NAME"),
version = env ! ("CARGO_PKG_VERSION"),
author = env ! ("CARGO_PKG_AUTHORS"),
about = env ! ("CARGO_PKG_DESCRIPTION"),
arg_required_else_help = true,
)]
struct Args {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[clap(arg_required_else_help = true)]
    MakeSystemDict(MakeSystemDictArgs),
    #[clap(arg_required_else_help = true)]
    MakeSystemLanguageModel(MakeSystemLanguageModelArgs),
}

#[derive(Debug, clap::Args)]
/// text のファイルからシステム辞書ファイルを作成する。
/// 入力元となるファイルは以下のような形式である。
/// UTF-8 でエンコードされたプレインテキストで、各行によみがなと漢字が半角空白区切りでおさめられている。
/// 漢字は slash でくぎられて複数格納されている。
///
/// ```
///     みやがく 宮学
///     みやがた 宮方/宮形/宮型
/// ```
///
struct MakeSystemDictArgs {
    /// 生成元のテキストファイル
    txtfile: String,
    /// 出力先のトライが格納されるファイル
    triefile: String,
}

/// システム言語モデルを生成する。
#[derive(Debug, clap::Args)]
struct MakeSystemLanguageModelArgs {
    unigram_src: String,
    unigram_dst: String,
    bigram_src: String,
    bigram_dst: String,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    match args.command {
        Commands::MakeSystemDict(opt) => make_system_dict(&opt.txtfile, &opt.triefile),
        Commands::MakeSystemLanguageModel(opt) => make_system_lm(
            &opt.unigram_src,
            &opt.unigram_dst,
            &opt.bigram_src,
            &opt.bigram_dst,
        ),
    }
}
