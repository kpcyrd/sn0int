#!/bin/sh
set -exu
case "$1" in
    linux)
        sudo apt update
        sudo apt install libsqlite3-dev libseccomp-dev libsodium-dev
        ;;
    osx)
        brew install libsodium
        ;;
esac
