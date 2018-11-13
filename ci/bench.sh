#!/bin/sh
set -eu
X=$(mktemp -d)
cd "$X"

mkdir -p "$X/.cache"
cp -r "$HOME/.cache/sn0int" "$X/.cache/"

#export CARGO_HOME="${CARGO_HOME:-$HOME/.cargo}"
#export RUSTUP_HOME="${RUSTUP_HOME:-$HOME/.rustup}"
export HOME="$X"

cat > 1k.lua <<EOF
-- Description: Insert 1k random subdomains
-- Version: 0.1.0
-- Source: domains
-- License: GPL-3.0

function run(arg)
    for i=1,1000 do
        x = 'foo' .. i .. '.example.com'

        db_add('subdomain', {
            domain_id=arg['id'],
            value=x,
        })
    end
end
EOF

echo '[*] Setting up workspace'
echo 'add domain example.com' | "$@" > /dev/null
echo '[*] Running 1k inserts'
time "$@" run -f ./1k.lua
