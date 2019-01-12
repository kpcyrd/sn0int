#!/bin/sh
set -exu
case "$1" in
    linux)
        sudo apt update
        sudo apt install libsqlite3-dev libseccomp-dev
        ;;
    windows)
        curl -fsS --retry 3 --retry-connrefused -o sqlite3.zip https://sqlite.org/2017/sqlite-dll-win64-x64-3160200.zip
        7z e sqlite3.zip -y
        "C:\\Program Files (x86)\\Microsoft Visual Studio 14.0\\VC\\bin\\lib.exe" /def:sqlite3.def /OUT:sqlite3.lib /machine:x64
        ;;
esac
