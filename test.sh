#!/usr/bin/env bash
set -eux
source ~/.cargo/env

echo -e "Test SSI output is complied with JSON format ..."
cargo run -- -r 0x3b559d574c4a9f13e55d0256655f0f71a70a703766226f1080f80022e39c057d -P System -F Account  ./db | jq
echo -e "Test SSI sumarized output is complied with JSON format ..."
cargo run -- -r 0x3b559d574c4a9f13e55d0256655f0f71a70a703766226f1080f80022e39c057d -P System -F Account -s ./db  | jq
echo -e "\e[0;32m  +------------------+ \n\e[0;32m  | JSON Format Pass | \n\e[0;32m  +------------------+ \e[0m"

