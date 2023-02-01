# TUT 用のマッピングテーブルを生成する。
# Canna 用の定義ファイルから生成する。

use strict;
use warnings;

use LWP::UserAgent;
use Data::Dumper;
use Encode qw/decode/;

binmode STDIN,  ":utf8";
binmode STDOUT,  ":utf8";
binmode STDERR,  ":utf8";

my $ua = LWP::UserAgent->new();

my $url = 'https://crew-lab.sfc.keio.ac.jp/projects/tut/data/tut.kpdef';

my $res = $ua->get($url);
my @lines = split /\n/, decode('euc-jp', $res->content);
for my $line (@lines) {
    if ($line =~ /^(\S+)[\t ](\S+)$/) {
        my $rom = $1;
        my $surface = $2;

        print "  \"$rom\": \"$surface\"\n";
    }
}

