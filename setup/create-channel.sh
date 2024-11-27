#!/bin/bash
set -e

SOURCE=""
SOURCE_ID=""

HOME_DIR="$HOME/.hermes"
CONFIG_LOCATION="$HOME_DIR/config.toml"
MNEMONIC_SOURCE="$HOME/.hermes/mnemonics/common.txt"

err_report() {
    echo "Error on line $1"
}

trap 'err_report $LINENO' ERR

function usage {
  echo "Usage: $0 [-h] [-f file] [-s chain] [-r root]"
  echo "  -h          Display this help message"
  echo "  -f file     Specify the file with chains to be configured"
  echo "  -s chain    Specify the chain which will be used as source for packet filtering"
  echo "  -r root     Specify config.toml root folder"
  exit 1
}

function add-key() {
  local chain_id=$1
  local bip_path=$2
  echo "Adding key for chain_id $chain_id"
  rm -rf ./keys/"$chain_id"/
  
  if [[ -z "$bip_path" || "${bip_path}" == "null" ]]; then
    hermes --config "$CONFIG_LOCATION" keys add --chain "$chain_id" --mnemonic-file "$MNEMONIC_SOURCE"
  else
    hermes --config "$CONFIG_LOCATION" keys add --chain "$chain_id" --mnemonic-file "$MNEMONIC_SOURCE" --hd-path "m/44'/$bip_path'/0'/0/0"
  fi
}

# Uses chain
function auto-config() {
  local chain_id
  chain_id=$(echo "$1" | tr '[:upper:]' '[:lower:]')
  echo "Auto config for chains $SOURCE - $chain"
  hermes config auto --only "$SOURCE" --output "$CONFIG_LOCATION" --chains "$SOURCE":key-"$SOURCE_ID" "$chain":key-"$chain_id"
}

# Uses chain_id
function channels-exist {
  local chain_id=$1
  echo "===>  Checking channel $SOURCE_ID => $chain_id"
  hermes --config "$CONFIG_LOCATION" channels query --source "$SOURCE_ID" --dest "$chain_id" --status
}

# Uses chain_id
function create-channel() {
  local chain_id=$1
  echo "===>  Creating channel $SOURCE_ID => $chain_id"
  hermes --config "$CONFIG_LOCATION" channels create --source "$SOURCE_ID" --dest "$chain_id" --a-port transfer --b-port transfer
}

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

function update_max_gas() {
  sed -i -s 's/max_gas = 400000/max_gas = 1000000/g' "$CONFIG_LOCATION"
}

function update_terra_rpc() {
  sed -i -s 's/terra-mainnet-rpc.autostake.com/terra-rpc.polkachu.com/g' "$CONFIG_LOCATION"
  sed -i -s 's/terra-rpc.lavenderfive.com/terra-rpc.polkachu.com/g' "$CONFIG_LOCATION"
}

function enable-all() {
  sed -i -s 's/enabled = false/enabled = true/g' "$CONFIG_LOCATION"
}

function health-check() {
  hermes health-check
}

function check-balance() {
  echo "Checking balance for chain $1"
  hermes keys balance --chain "$1"
}


# -- Parse arguments ------------------------------------

while getopts "hf:s:r:" opt; do
  case $opt in
    h)
      usage
      ;;
    f)
      source_file=$OPTARG
      ;;
    s)
      source_chain=$OPTARG
      ;;
    r)
      root=$OPTARG
      ;;
    \?)
      echo "Invalid option: -$OPTARG" >&2
      usage
      ;;
  esac
done

if [ -z "$source_file" ]; then
  echo "Source file file not specified"
  usage
fi

if [ -z "$source_chain" ]; then
  echo "Source chain is not specified"
  usage
fi

if [ -z "$root" ]; then
  echo "Home folder is not specified, using default: $HOME_DIR"
else
  HOME_DIR=$(readlink -f "$root")
  echo "Home folder is specified: $HOME_DIR"
fi

# -- Setting variables -----------------------------------------------

CONFIG_LOCATION="$HOME_DIR/config.toml"
echo "Config location: $CONFIG_LOCATION"

SOURCE=$source_chain

if [[ -z "$SOURCE" || "${SOURCE}" == "null" ]]; then
  echo "Source chain is missing in arguments"
  usage;
fi

# -- Main -----------------------------------------------

SOURCE=$(echo "$SOURCE" | tr '[:upper:]' '[:lower:]')
SOURCE_ID=$(chainid "$SOURCE")
SOURCE_bip44_index=$(bip44_path "$SOURCE")

echo "-------------------------------------------------------------------------"
echo "Source chain: $SOURCE; id = $SOURCE_ID; bip44_index = $SOURCE_bip44_index"
echo "-------------------------------------------------------------------------"

# Read the file line by line and execute your 1 command for each line
while read -r line; do
  if [[ "$line" == "" ]]; then
    echo "Skipping empty line"
    continue
  fi

  chain=$(echo "$line" | tr '[:upper:]' '[:lower:]')

  if [[ "$chain" == "$SOURCE" ]]; then
      echo "Skipping source chain $chain"
      continue
  fi

  chain_id=$(chainid "$chain")
  bip44_index=$(bip44_path "$chain")

  echo "-------------------------------------------------------------------------"
  echo "** Started creating channels from $SOURCE to $chain"
  echo "** Chain: $chain"
  echo "** Chain ID: $chain_id"
  echo "** Bip44 Index: $bip44_index"
  echo "-------------------------------------------------------------------------"

  auto-config "$chain_id" || true

  if channels-exist "$chain_id"; then
    echo "Channel already exists for chain $chain_id. Skipping..."
    continue
  fi

  hermes config endpoints || true
  enable-all || true
  update_max_gas || true
  health-check || true
  add-key "$chain_id" "$bip44_index"  || true
  add-key "$SOURCE_ID" "$SOURCE_bip44_index"  || true

  check-balance "$SOURCE_ID" || true
  check-balance "$chain_id" || true
  create-channel "$chain_id" || true
done < "$source_file"
