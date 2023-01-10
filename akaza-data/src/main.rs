use std::io::Write;

use clap::{Parser, Subcommand};

use crate::subcmd::check::check;
use crate::subcmd::evaluate::evaluate;
use crate::subcmd::make_stats_system_unigram_lm::make_stats_system_unigram_lm;
use crate::subcmd::make_system_lm::make_system_lm;
use crate::subcmd::make_text_dict::{make_single_term, make_system_dict};
use crate::subcmd::structured_perceptron::learn_structured_perceptron;
use crate::subcmd::tokenize::tokenize;
use crate::subcmd::vocab::vocab;
use crate::subcmd::wfreq::wfreq;

mod subcmd;
mod tokenizer;
mod utils;
mod wikipedia;

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
    MakeSingleTerm(MakeSingleTermArgs),
    #[clap(arg_required_else_help = true)]
    MakeSystemLanguageModel(MakeSystemLanguageModelArgs),
    #[clap(arg_required_else_help = true)]
    Evaluate(EvaluateArgs),
    #[clap(arg_required_else_help = true)]
    Check(CheckArgs),
    LearnStructuredPerceptron(LearnStructuredPerceptronArgs),
    Tokenize(TokenizeArgs),
    Wfreq(WfreqArgs),
    Vocab(VocabArgs),
    MakeStatsSystemUnigramLM(MakeStatsSystemUnigramLMArgs),
}

#[derive(Debug, clap::Args)]
/// システム辞書ファイルを作成する。
struct MakeSystemDictArgs {
    #[arg(short, long)]
    vocab_file: String,
    /// デバッグのための中間テキストファイル
    txt_file: String,
    /// 出力先のトライが格納されるファイル
    trie_file: String,
}

/// 単項辞書を作成する
#[derive(Debug, clap::Args)]
struct MakeSingleTermArgs {
    txt_file: String,
    trie_file: String,
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
    src_dir: String,
}

/// コーパスを形態素解析機でトーカナイズする
#[derive(Debug, clap::Args)]
struct TokenizeArgs {
    #[arg(short, long)]
    tokenizer: String,
    #[arg(short, long)]
    user_dict: Option<String>,
    src_dir: String,
    dst_dir: String,
}

#[derive(Debug, clap::Args)]
struct WfreqArgs {
    src_dir: String,
    dst_file: String,
}

#[derive(Debug, clap::Args)]
struct VocabArgs {
    /// 語彙ファイルに収録する単語数のあしきりライン。
    /// 増やすと辞書ファイルサイズが大きくなり、実行時のメモリ使用量も増大する。
    /// 増やすと変換可能な語彙が増える。
    #[arg(short, long)]
    threshold: u32,
    src_file: String,
    dst_file: String,
}

/// ユニグラム言語モデルを作成する。
#[derive(Debug, clap::Args)]
struct MakeStatsSystemUnigramLMArgs {
    src_file: String,
    dst_file: String,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    env_logger::Builder::new()
        .filter_level(args.verbose.log_level_filter())
        .format(|buf, record| {
            let ts = buf.timestamp_micros();
            // show thread id
            writeln!(
                buf,
                "{}: {:?}: {}: {}",
                ts,
                std::thread::current().id(),
                buf.default_level_style(record.level())
                    .value(record.level()),
                record.args()
            )
        })
        .init();

    match args.command {
        Commands::MakeSystemDict(opt) => {
            make_system_dict(&opt.txt_file, &opt.trie_file, Some(opt.vocab_file.as_str()))
        }
        Commands::MakeSingleTerm(opt) => make_single_term(&opt.txt_file, &opt.trie_file),
        Commands::MakeSystemLanguageModel(opt) => make_system_lm(
            &opt.unigram_src,
            &opt.unigram_dst,
            &opt.bigram_src,
            &opt.bigram_dst,
        ),
        Commands::Evaluate(opt) => evaluate(&opt.corpus_dir, &opt.system_data_dir),
        Commands::Check(opt) => check(&opt.yomi),
        Commands::LearnStructuredPerceptron(opts) => {
            learn_structured_perceptron(&opts.src_dir, opts.epochs)
        }
        Commands::Tokenize(opt) => tokenize(
            opt.tokenizer.as_str(),
            opt.user_dict,
            opt.src_dir.as_str(),
            opt.dst_dir.as_str(),
        ),
        Commands::Wfreq(opt) => wfreq(opt.src_dir.as_str(), opt.dst_file.as_str()),
        Commands::Vocab(opt) => vocab(opt.src_file.as_str(), opt.dst_file.as_str(), opt.threshold),
        Commands::MakeStatsSystemUnigramLM(opt) => {
            make_stats_system_unigram_lm(opt.src_file.as_str(), opt.dst_file.as_str())
        }
    }
}
