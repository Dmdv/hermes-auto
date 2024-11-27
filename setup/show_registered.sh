#!/bin/bash
set -e

SOURCE_ID="phoenix-1"

# Returns chain_id
function chainid() {
  local chain
  local chain_id

  chain=$(echo "$1" | tr '[:upper:]' '[:lower:]')
  chain_id=$(curl -s -X  GET "https://raw.githubusercontent.com/cosmos/chain-registry/master/$chain/chain.json" | jq -r '.|.chain_id')
  echo "$chain_id"
}

function bip44_path() {
  local chain
  local slip44

  chain=$(echo "$1" | tr '[:upper:]' '[:lower:]')
  slip44=$(curl -s -X  GET "https://raw.githubusercontent.com/cosmos/chain-registry/master/$chain/chain.json" | jq -r '.|.slip44')
  echo "$slip44" || ""
}

function show-channels() {
  curl -s -X GET "https://ibc.tfm.com/channels/pairs?sourceChainId=$1&destinationChainId=$2&page=1&take=1000" | jq -r '.data[] | .channelId + " -> " + .counterpartyChannelId + " " +"(" + .baseDenom.symbol + ")"'
}

while read -r line; do
  if [[ "$line" == "" ]]; then
    echo "Skipping empty line"
    continue
  fi

  chain=$line

  chain_id1=$SOURCE_ID
  chain_id2=$(chainid "$chain")

  echo "======================================"
  echo "TERRA -> : $chain_id2"
  echo "======================================"

  show-channels "$chain_id1" "$chain_id2" || true

done < "$1"



