#!/bin/bash
mkdir -p ~/rust-installer
curl -sL https://static.rust-lang.org/rustup.sh -o ~/rust-installer/rustup.sh
export RUSTUP_PREFIX=~/rust
sh ~/rust-installer/rustup.sh --spec=stable --disable-sudo -y
