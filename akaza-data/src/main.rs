use clap::{Parser, Subcommand};

use crate::subcmd::make_system_dict::make_system_dict;

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
    txtfile: String,
    triefile: String,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    match args.command {
        Commands::MakeSystemDict(opt) => make_system_dict(&opt.txtfile, &opt.triefile),
    }
}
