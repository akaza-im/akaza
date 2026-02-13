use std::io::Write;

use clap::{Parser, Subcommand, ValueEnum};

#[derive(Debug, Clone, ValueEnum)]
enum OutputFormat {
    Text,
    Json,
}

use libakaza::graph::reranking::ReRankingWeights;

use crate::subcmd::bench::{bench, BenchOptions};
use crate::subcmd::check::{check, CheckOptions};
use crate::subcmd::convert_skip_bigram_model::convert_skip_bigram_model;
use crate::subcmd::dump_bigram_dict::dump_bigram_dict;
use crate::subcmd::dump_unigram_dict::dump_unigram_dict;
use crate::subcmd::evaluate::evaluate;
use crate::subcmd::learn_corpus::learn_corpus;
use crate::subcmd::make_dict::make_system_dict;
use crate::subcmd::make_stats_system_bigram_lm::make_stats_system_bigram_lm;
use crate::subcmd::make_stats_system_skip_bigram_lm::make_stats_system_skip_bigram_lm;
use crate::subcmd::make_stats_system_unigram_lm::make_stats_system_unigram_lm;
use crate::subcmd::tokenize::tokenize;
use crate::subcmd::tokenize_line::tokenize_line;
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
    TokenizeLine(TokenizeLineArgs),

    Wfreq(WfreqArgs),
    Vocab(VocabArgs),

    #[clap(arg_required_else_help = true)]
    MakeDict(MakeDictArgs),

    WordcntUnigram(WordcntUnigramArgs),
    #[clap(arg_required_else_help = true)]
    WordcntBigram(WordcntBigramArgs),
    #[clap(arg_required_else_help = true)]
    WordcntSkipBigram(WordcntSkipBigramArgs),

    LearnCorpus(LearnCorpusArgs),

    #[clap(arg_required_else_help = true)]
    Check(CheckArgs),
    #[clap(arg_required_else_help = true)]
    Evaluate(EvaluateArgs),

    Bench(BenchArgs),

    DumpUnigramDict(DumpUnigramDictArgs),
    DumpBigramDict(DumpBigramDictArgs),

    /// wordcnt skip-bigram trie → skip_bigram.model に変換
    #[clap(arg_required_else_help = true)]
    ConvertSkipBigramModel(ConvertSkipBigramModelArgs),
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

/// 一行の自然文をトーカナイズする
#[derive(Debug, clap::Args)]
struct TokenizeLineArgs {
    #[arg(short, long)]
    user_dict: Option<String>,
    #[arg(short, long)]
    system_dict: String,
    #[arg(long)]
    kana_preferred: bool,
    text: Option<String>,
}

/// トーカナイズされたコーパスから単語頻度ファイルを生成する
#[derive(Debug, clap::Args)]
struct WfreqArgs {
    #[arg(long)]
    src_dir: Vec<String>,
    dst_file: String,
}

/// 単語頻度ファイルから語彙リストを生成する
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

/// skip-bigram 言語モデルを生成する。
#[derive(Debug, clap::Args)]
struct WordcntSkipBigramArgs {
    #[arg(short, long)]
    threshold: u32,
    #[arg(long)]
    corpus_dirs: Vec<String>,
    unigram_trie_file: String,
    skip_bigram_trie_file: String,
}

/// コーパスから言語モデルを学習する
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
    /// skip-bigram のソースファイル（省略時は skip-bigram 学習をスキップ）
    #[arg(long)]
    src_skip_bigram: Option<String>,
    /// skip-bigram の出力ファイル（--src-skip-bigram 指定時は必須）
    #[arg(long)]
    dst_skip_bigram: Option<String>,
}

/// かな漢字変換を実行する（CLI テスト用）
#[derive(Debug, clap::Args)]
struct CheckArgs {
    /// 変換したい読みがな（省略時は stdin から行ごとに読み取る）
    yomi: Option<String>,
    /// 期待する変換結果（指定すると DOT グラフを出力）
    expected: Option<String>,
    /// ユーザーデータ（学習データ）を使用する
    #[arg(short, long, default_value_t = false)]
    user_data: bool,
    /// 出力形式
    #[arg(short, long, value_enum, default_value_t = OutputFormat::Text)]
    format: OutputFormat,
    /// 各文節の候補数
    #[arg(short = 'n', long, default_value_t = 1)]
    candidates: usize,
    /// UTF-8 辞書ファイル（設定ファイルの辞書に追加）
    #[arg(long)]
    utf8_dict: Vec<String>,
    /// EUC-JP 辞書ファイル（設定ファイルの辞書に追加）
    #[arg(long)]
    eucjp_dict: Vec<String>,
    /// モデルデータの格納ディレクトリ（省略時は設定ファイルから読み込む）
    #[arg(short, long)]
    model_dir: Option<String>,
    /// k-best 分節パターン数（指定すると上位 k 個の分節パターンを表示）
    #[arg(short, long)]
    k_best: Option<usize>,
    /// リランキング: 既知 bigram コストの重み
    #[arg(long, default_value_t = 1.0)]
    bigram_weight: f32,
    /// リランキング: トークン長ペナルティの重み
    #[arg(long, default_value_t = 2.0)]
    length_weight: f32,
    /// リランキング: 未知 bigram フォールバックコストの重み
    #[arg(long, default_value_t = 1.0)]
    unknown_bigram_weight: f32,
    /// リランキング: skip-bigram コストの重み
    #[arg(long, default_value_t = 0.2)]
    skip_bigram_weight: f32,
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
    /// k-best 評価（上位 k 個のパスに正解が含まれるか判定）
    #[arg(long, default_value_t = 5)]
    k_best: usize,
    /// リランキング: 既知 bigram コストの重み
    #[arg(long, default_value_t = 1.0)]
    bigram_weight: f32,
    /// リランキング: トークン長ペナルティの重み
    #[arg(long, default_value_t = 2.0)]
    length_weight: f32,
    /// リランキング: 未知 bigram フォールバックコストの重み
    #[arg(long, default_value_t = 1.0)]
    unknown_bigram_weight: f32,
    /// リランキング: skip-bigram コストの重み
    #[arg(long, default_value_t = 0.2)]
    skip_bigram_weight: f32,
}

/// インクリメンタル変換のベンチマーク
#[derive(Debug, clap::Args)]
struct BenchArgs {
    #[arg(long)]
    corpus: Vec<String>,
    #[arg(long)]
    utf8_dict: Vec<String>,
    #[arg(long)]
    eucjp_dict: Vec<String>,
    /// モデルデータの格納ディレクトリ（省略時は設定ファイルから読み込む）
    #[arg(long)]
    model_dir: Option<String>,
    /// ベンチマーク対象の最大文数
    #[arg(long, default_value_t = 100)]
    max_sentences: usize,
    /// k-best のパス数
    #[arg(short, long, default_value_t = 5)]
    k: usize,
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

/// wordcnt skip-bigram trie を skip_bigram.model に変換する
#[derive(Debug, clap::Args)]
struct ConvertSkipBigramModelArgs {
    /// 入力: wordcnt skip-bigram trie ファイル
    src_skip_bigram: String,
    /// 入力: wordcnt unigram trie ファイル（旧 word_id のソース）
    src_wordcnt_unigram: String,
    /// 入力: unigram.model ファイル（新 word_id のソース）
    dst_unigram_model: String,
    /// 出力: skip_bigram.model ファイル
    dst: String,
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
                record.level(),
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
        Commands::TokenizeLine(opt) => tokenize_line(
            opt.system_dict.as_str(),
            opt.user_dict,
            opt.kana_preferred,
            opt.text,
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
        Commands::WordcntSkipBigram(opt) => make_stats_system_skip_bigram_lm(
            opt.threshold,
            &opt.corpus_dirs,
            &opt.unigram_trie_file,
            &opt.skip_bigram_trie_file,
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
            opts.src_skip_bigram.as_deref(),
            opts.dst_unigram.as_str(),
            opts.dst_bigram.as_str(),
            opts.dst_skip_bigram.as_deref(),
        ),
        Commands::Check(opt) => check(CheckOptions {
            yomi: opt.yomi,
            expected: opt.expected,
            use_user_data: opt.user_data,
            eucjp_dict: &opt.eucjp_dict,
            utf8_dict: &opt.utf8_dict,
            model_dir: opt.model_dir.as_deref(),
            json_output: matches!(opt.format, OutputFormat::Json),
            num_candidates: opt.candidates,
            k_best: opt.k_best,
            reranking_weights: ReRankingWeights {
                bigram_weight: opt.bigram_weight,
                length_weight: opt.length_weight,
                unknown_bigram_weight: opt.unknown_bigram_weight,
                skip_bigram_weight: opt.skip_bigram_weight,
            },
        }),
        Commands::Evaluate(opt) => evaluate(
            &opt.corpus,
            &opt.eucjp_dict,
            &opt.utf8_dict,
            opt.model_dir,
            opt.k_best,
            ReRankingWeights {
                bigram_weight: opt.bigram_weight,
                length_weight: opt.length_weight,
                unknown_bigram_weight: opt.unknown_bigram_weight,
                skip_bigram_weight: opt.skip_bigram_weight,
            },
        ),
        Commands::Bench(opt) => bench(BenchOptions {
            corpus: &opt.corpus,
            eucjp_dict: &opt.eucjp_dict,
            utf8_dict: &opt.utf8_dict,
            model_dir: opt.model_dir.as_deref(),
            max_sentences: opt.max_sentences,
            k: opt.k,
        }),
        Commands::DumpUnigramDict(opt) => dump_unigram_dict(opt.dict.as_str()),
        Commands::DumpBigramDict(opt) => {
            dump_bigram_dict(opt.unigram_file.as_str(), opt.bigram_file.as_str())
        }
        Commands::ConvertSkipBigramModel(opt) => convert_skip_bigram_model(
            opt.src_skip_bigram.as_str(),
            opt.src_wordcnt_unigram.as_str(),
            opt.dst_unigram_model.as_str(),
            opt.dst.as_str(),
        ),
    }
}
