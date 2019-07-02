#!/usr/bin/env perl
use strict; use warnings;

my ($readme, $toc) = @ARGV;
defined $readme or die 'missing readme path';
defined $toc or die 'missing toc path';

open(my $r, "<$readme") or die 'failed to open readme';
open(my $t, "<$toc") or die 'failed to open toc';

my $re = qr/^\s*- \[.+\]\(https:\/\/sn0int.readthedocs.io\/en\/.+\)$/;

# pass through start of readme
while (<$r>) {
    last if ($_ =~ $re);
    print $_;
}

# skip toc
while (<$r>) {
    last unless ($_ =~ $re);
}

# generate new toc
while (my $line = <$t>) {
    if ($line =~ /toctree-l(\d).*href="([^"]+)">(.+)<\/a/) {
        my $space = int($1)-1;
        my $section = $2;
        my $label = $3;
        $label =~ s/([\[\]])/\\$1/g;
        print " " x ($space*2), "- [$label](https://sn0int.readthedocs.io/en/latest/$section)\n";
    }
}
print;

# pass through end of readme
while (<$r>) {
    print $_;
}
