#!/usr/bin/env bash

# Start ore mining private pool client

set -e

CLI=$HOME/miner/ore-private-pool-cli/target/release/ore-ppl-cli

MKP="$HOME/.config/solana/id.json"

CORES=8

# The command you want to run

# CMD="$HOME/miner/ore-private-pool-cli/target/release/ore-ppl-cli \
#         --use-http \
#         --url 192.168.xx.xx:3000 \
#         --keypair $MKP \
#         mine --cores 8

CMD="$CLI \
        --use-http \
        --url 192.168.xx.xx:3000 \
        --keypair $MKP \
        protomine --cores $CORES"

echo $CMD
until bash -c "$CMD"; do
    echo "Starting client command failed. Restart..."
    sleep 2
done
