use clap::{Parser, Subcommand};

use crate::subcmd::check::check;
use crate::subcmd::evaluate::evaluate;
use crate::subcmd::make_system_dict::make_system_dict;
use crate::subcmd::make_system_lm::make_system_lm;
use crate::subcmd::make_text_dict::make_text_dict;
use crate::subcmd::structured_perceptron::learn_structured_perceptron;
use crate::subcmd::vibrato_annotate::annotate_wikipedia;

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
    #[clap(flatten)]
    verbose: clap_verbosity_flag::Verbosity,

    #[clap(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[clap(arg_required_else_help = true)]
    MakeSystemDict(MakeSystemDictArgs),
    #[clap(arg_required_else_help = true)]
    MakeSystemLanguageModel(MakeSystemLanguageModelArgs),
    #[clap(arg_required_else_help = true)]
    Evaluate(EvaluateArgs),
    #[clap(arg_required_else_help = true)]
    Check(CheckArgs),
    LearnStructuredPerceptron(LearnStructuredPerceptronArgs),
    MakeTextDict(MakeTextDictArgs),
    VibratoAnnotate(VibratoAnnotateArgs),
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

/// 変換精度を評価する
#[derive(Debug, clap::Args)]
struct EvaluateArgs {
    /// コーパスが格納されているディレクトリ
    corpus_dir: String,
    /// 評価に利用するシステムデータのディレクトリ
    system_data_dir: String,
}

/// 動作確認する
#[derive(Debug, clap::Args)]
struct CheckArgs {
    /// 変換したい読みがな
    yomi: String,
}

/// 動作確認する
#[derive(Debug, clap::Args)]
struct LearnStructuredPerceptronArgs {
    #[arg(short, long, default_value_t = 10)]
    epochs: i32,
}

/// テキスト辞書を作る
#[derive(Debug, clap::Args)]
struct MakeTextDictArgs {}

/// Wikipedia を Vibrato でアノテーションする
#[derive(Debug, clap::Args)]
struct VibratoAnnotate {}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    env_logger::Builder::new()
        .filter_level(args.verbose.log_level_filter())
        .init();

    match args.command {
        Commands::MakeSystemDict(opt) => make_system_dict(&opt.txtfile, &opt.triefile),
        Commands::MakeSystemLanguageModel(opt) => make_system_lm(
            &opt.unigram_src,
            &opt.unigram_dst,
            &opt.bigram_src,
            &opt.bigram_dst,
        ),
        Commands::Evaluate(opt) => evaluate(&opt.corpus_dir, &opt.system_data_dir),
        Commands::Check(opt) => check(&opt.yomi),
        Commands::LearnStructuredPerceptron(opts) => learn_structured_perceptron(opts.epochs),
        Commands::MakeTextDict(_) => make_text_dict(),
        Commands::VibratoAnnotate(_) => annotate_wikipedia(),
    }
}
