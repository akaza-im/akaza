extern crate core;

use std::io::Write;

use clap::{Parser, Subcommand};

use crate::subcmd::check::check;
use crate::subcmd::dump_bigram_dict::dump_bigram_dict;
use crate::subcmd::dump_unigram_dict::dump_unigram_dict;
use crate::subcmd::evaluate::evaluate;
use crate::subcmd::learn_corpus::learn_corpus;
use crate::subcmd::make_dict::make_system_dict;
use crate::subcmd::make_stats_system_bigram_lm::make_stats_system_bigram_lm;
use crate::subcmd::make_stats_system_unigram_lm::make_stats_system_unigram_lm;
use crate::subcmd::tokenize::tokenize;
use crate::subcmd::vocab::vocab;
use crate::subcmd::wfreq::wfreq;

mod corpus_reader;
mod subcmd;
mod tokenizer;
mod utils;
mod wordcnt;

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
    Tokenize(TokenizeArgs),

    Wfreq(WfreqArgs),
    Vocab(VocabArgs),

    #[clap(arg_required_else_help = true)]
    MakeDict(MakeDictArgs),

    WordcntUnigram(WordcntUnigramArgs),
    #[clap(arg_required_else_help = true)]
    WordcntBigram(WordcntBigramArgs),

    LearnCorpus(LearnCorpusArgs),

    #[clap(arg_required_else_help = true)]
    Check(CheckArgs),
    #[clap(arg_required_else_help = true)]
    Evaluate(EvaluateArgs),

    DumpUnigramDict(DumpUnigramDictArgs),
    DumpBigramDict(DumpBigramDictArgs),
}

/// コーパスを形態素解析機でトーカナイズする
#[derive(Debug, clap::Args)]
struct TokenizeArgs {
    #[arg(short, long)]
    reader: String,
    #[arg(short, long)]
    user_dict: Option<String>,
    #[arg(short, long)]
    system_dict: String,
    #[arg(long)]
    kana_preferred: bool,
    src_dir: String,
    dst_dir: String,
}

#[derive(Debug, clap::Args)]
struct WfreqArgs {
    #[arg(long)]
    src_dir: Vec<String>,
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

#[derive(Debug, clap::Args)]
/// システム辞書ファイルを作成する。
struct MakeDictArgs {
    #[arg(short, long)]
    corpus: Vec<String>,
    #[arg(short, long)]
    unidic: String,
    #[arg(long)]
    vocab: String,
    /// デバッグのための中間テキストファイル
    txt_file: String,
}

/// ユニグラム言語モデルを作成する。
#[derive(Debug, clap::Args)]
struct WordcntUnigramArgs {
    src_file: String,
    dst_file: String,
}

/// システム言語モデルを生成する。
#[derive(Debug, clap::Args)]
struct WordcntBigramArgs {
    #[arg(short, long)]
    threshold: u32,
    #[arg(long)]
    corpus_dirs: Vec<String>,
    unigram_trie_file: String,
    bigram_trie_file: String,
}

/// 動作確認する
#[derive(Debug, clap::Args)]
struct LearnCorpusArgs {
    #[arg(short, long)]
    delta: u32,
    #[arg(long, default_value_t = 10)]
    may_epochs: i32,
    #[arg(long, default_value_t = 100)]
    should_epochs: i32,
    #[arg(long, default_value_t = 1000)]
    must_epochs: i32,
    may_corpus: String,
    should_corpus: String,
    must_corpus: String,
    src_unigram: String,
    src_bigram: String,
    dst_unigram: String,
    dst_bigram: String,
}

/// 動作確認する
#[derive(Debug, clap::Args)]
struct CheckArgs {
    #[arg(short, long, default_value_t = false)]
    user_data: bool,
    /// 変換したい読みがな
    yomi: String,
    expected: Option<String>,
    #[arg(long)]
    utf8_dict: Vec<String>,
    #[arg(long)]
    eucjp_dict: Vec<String>,
    #[arg(long)]
    model_dir: String,
}

/// 変換精度を評価する
#[derive(Debug, clap::Args)]
struct EvaluateArgs {
    #[arg(long)]
    corpus: Vec<String>,
    #[arg(long)]
    utf8_dict: Vec<String>,
    #[arg(long)]
    eucjp_dict: Vec<String>,
    #[arg(long)]
    model_dir: String,
}

/// ユニグラム辞書ファイルをダンプする
#[derive(Debug, clap::Args)]
struct DumpUnigramDictArgs {
    dict: String,
}

/// バイグラム辞書ファイルをダンプする
#[derive(Debug, clap::Args)]
struct DumpBigramDictArgs {
    unigram_file: String,
    bigram_file: String,
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
        Commands::Tokenize(opt) => tokenize(
            opt.reader,
            opt.system_dict,
            opt.user_dict,
            opt.kana_preferred,
            opt.src_dir.as_str(),
            opt.dst_dir.as_str(),
        ),
        Commands::Wfreq(opt) => wfreq(&opt.src_dir, opt.dst_file.as_str()),
        Commands::Vocab(opt) => vocab(opt.src_file.as_str(), opt.dst_file.as_str(), opt.threshold),
        Commands::MakeDict(opt) => make_system_dict(
            &opt.txt_file,
            Some(opt.vocab.as_str()),
            opt.corpus,
            opt.unidic,
        ),
        Commands::WordcntBigram(opt) => make_stats_system_bigram_lm(
            opt.threshold,
            &opt.corpus_dirs,
            &opt.unigram_trie_file,
            &opt.bigram_trie_file,
        ),
        Commands::WordcntUnigram(opt) => {
            make_stats_system_unigram_lm(opt.src_file.as_str(), opt.dst_file.as_str())
        }
        Commands::LearnCorpus(opts) => learn_corpus(
            opts.delta,
            opts.may_epochs,
            opts.should_epochs,
            opts.must_epochs,
            opts.may_corpus.as_str(),
            opts.should_corpus.as_str(),
            opts.must_corpus.as_str(),
            opts.src_unigram.as_str(),
            opts.src_bigram.as_str(),
            opts.dst_unigram.as_str(),
            opts.dst_bigram.as_str(),
        ),
        Commands::Check(opt) => check(
            &opt.yomi,
            opt.expected,
            opt.user_data,
            &opt.eucjp_dict,
            &opt.utf8_dict,
            &opt.model_dir,
        ),
        Commands::Evaluate(opt) => {
            evaluate(&opt.corpus, &opt.eucjp_dict, &opt.utf8_dict, opt.model_dir)
        }
        Commands::DumpUnigramDict(opt) => dump_unigram_dict(opt.dict.as_str()),
        Commands::DumpBigramDict(opt) => {
            dump_bigram_dict(opt.unigram_file.as_str(), opt.bigram_file.as_str())
        }
    }
}
