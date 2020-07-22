#!/usr/bin/env bash
set -eux
source ~/.cargo/env

echo -e "Test SSI output is complied with JSON format ..."
cargo run -- -r 0x3b559d574c4a9f13e55d0256655f0f71a70a703766226f1080f80022e39c057d -P System ./db | jq
cargo run -- -r 0x3b559d574c4a9f13e55d0256655f0f71a70a703766226f1080f80022e39c057d -P System -F Account ./db | jq
cargo run -- -r 0x3b559d574c4a9f13e55d0256655f0f71a70a703766226f1080f80022e39c057d -P Balances ./db | jq
cargo run -- -r 0x3b559d574c4a9f13e55d0256655f0f71a70a703766226f1080f80022e39c057d -P Balances -F Account ./db | jq
echo -e "Test SSI sumarized output is complied with JSON format ..."
cargo run -- -r 0x3b559d574c4a9f13e55d0256655f0f71a70a703766226f1080f80022e39c057d -P System -s ./db  | jq
cargo run -- -r 0x3b559d574c4a9f13e55d0256655f0f71a70a703766226f1080f80022e39c057d -P System -F Account -s ./db  | jq
cargo run -- -r 0x3b559d574c4a9f13e55d0256655f0f71a70a703766226f1080f80022e39c057d -P Balances ./db -s | jq
cargo run -- -r 0x3b559d574c4a9f13e55d0256655f0f71a70a703766226f1080f80022e39c057d -P Balances -F Account ./db -s | jq
echo -e "Test SSI Get value from HashMap Field ..."
cargo run -- -r 0x3b559d574c4a9f13e55d0256655f0f71a70a703766226f1080f80022e39c057d -P Balances -F Account -T //Feride ./db
cargo run -- -r 0x940a55c41ce61b2d771e82f8a6c6f4939a712a644502f5efa7c59afea0a3a67e -P Balances -F Account -T '//Feride' ./db
echo -e "\e[0;32m  +------------------+ \n\e[0;32m  | JSON Format Pass | \n\e[0;32m  +------------------+ \e[0m"
