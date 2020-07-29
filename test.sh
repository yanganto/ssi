#!/usr/bin/env bash
# set -eux
# source ~/.cargo/env

echo -e "Test SSI output is complied with JSON format ..."
# cargo run -- -r 0x3b559d574c4a9f13e55d0256655f0f71a70a703766226f1080f80022e39c057d -P System ./db  | jq '.[]|keys'
cargo run -- -r 0x3b559d574c4a9f13e55d0256655f0f71a70a703766226f1080f80022e39c057d -P System -F Account ./db | jq '.[]|keys'
# cargo run -- -r 0x3b559d574c4a9f13e55d0256655f0f71a70a703766226f1080f80022e39c057d -P Balance ./db | jq '.[]|keys'
cargo run -- -r 0x3b559d574c4a9f13e55d0256655f0f71a70a703766226f1080f80022e39c057d -P Balance -F Account ./db | jq '.[]|keys'

echo -e "Test SSI sumarized output is complied with JSON format ..."

# TODO: There are half word issue in the test
# cargo run -- -r 0x3b559d574c4a9f13e55d0256655f0f71a70a703766226f1080f80022e39c057d -P System -s ./db  | jq '.[]|keys'
cargo run -- -r 0x3b559d574c4a9f13e55d0256655f0f71a70a703766226f1080f80022e39c057d -P System -F Account -s ./db  | jq '.[]|keys'
# cargo run -- -r 0x3b559d574c4a9f13e55d0256655f0f71a70a703766226f1080f80022e39c057d -P Balance ./db -s | jq '.[]|keys'
# cargo run -- -r 0x3b559d574c4a9f13e55d0256655f0f71a70a703766226f1080f80022e39c057d -P Balance -F Account ./db -s | jq '.[]|keys'

# echo -e "Test SSI Get value from HashMap Field ..."
cargo run -- -r 0x3b559d574c4a9f13e55d0256655f0f71a70a703766226f1080f80022e39c057d -P Balance -F Account -T //Feride ./db | jq '.[]|keys'
cargo run -- -r 0x940a55c41ce61b2d771e82f8a6c6f4939a712a644502f5efa7c59afea0a3a67e -P Balance -F Account -T '//Feride' ./db | jq '.[]|keys'

# TODO: too early to run into leaf
# echo -e "Test SSI semantic feature.."
# cargo run -- -r 0x940a55c41ce61b2d771e82f8a6c6f4939a712a644502f5efa7c59afea0a3a67e -P System -F Account -T '//Eve' ./db -s -e | jq '.[]|keys'

echo -e "Test storage key decode function.."
cargo run -- -d -k cec5070d609dd3497f72bde07fc96ba0e0cdd062e6eaf24295ad4ccfc41d4609ddddd
cargo run -- -d .test-data/block-499135.log
echo -e "\e[0;32m  +------------------+ \n\e[0;32m  | JSON Format Pass | \n\e[0;32m  +------------------+ \e[0m"
