#!/bin/bash
set -e

HOME_DIR="$HOME/.hermes"
CONFIG_LOCATION="$HOME/.hermes/config.toml"
MNEMONIC_SOURCE="$HOME/.hermes/mnemonics/common.txt"
REST_PORT=3000
METRIC_PORT=3001

err_report() {
    echo "Error on line $1"
}

trap 'err_report $LINENO' ERR

function usage {
  echo "Usage: $0 [-h] [-s source chain] [-d dest chain] [-r root] [-p port]  [-t port]"
  echo "  -h          Display this help message"
  echo "  -s chain    Specify the source chain"
  echo "  -d chain    Specify the destination chain"
  echo "  -r root     Specify config.toml root folder"
  echo "  -p port     Specify REST port"
  echo "  -t port     Specify TELEMETRY port"
  exit 1
}

function auto-config-with-source() {
  echo "*****************************************************************"
  echo "       Running auto-config with source $1 chain"
  echo "*****************************************************************"

  from="$1"
  shift
  array_of_args=("$@")
  echo "Adding chains to config file with source: $from and target:" "$@"
  hermes config auto --only "$from" --output "$CONFIG_LOCATION" --chains "${array_of_args[@]}"
}

function auto-config() {
  echo "*****************************************************************"
  echo "       Running auto-config"
  echo "*****************************************************************"

  echo "Adding chains to config file:" "$@"
  hermes config auto --output "$CONFIG_LOCATION" --chains "$@"
}

function update-max-gas() {
  sed -i -s 's/max_gas = 400000/max_gas = 1000000/g' "$CONFIG_LOCATION"
}

function update-terra-rpc() {
  sed -i -s 's/terra-mainnet-rpc.autostake.com/terra-rpc.polkachu.com/g' "$CONFIG_LOCATION"
  sed -i -s 's/terra-rpc.lavenderfive.com/terra-rpc.polkachu.com/g' "$CONFIG_LOCATION"
}

function update-ports() {
  sed -i -s "s/port = 3000/port = $REST_PORT/g" "$CONFIG_LOCATION"
  sed -i -s "s/port = 3001/port = $METRIC_PORT/g" "$CONFIG_LOCATION"
}

function enable-all() {
  sed -i -s 's/enabled = false/enabled = true/g' "$CONFIG_LOCATION"
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

function bip44_path() {
  local chain
  local slip44

  chain=$(echo "$1" | tr '[:upper:]' '[:lower:]')
  slip44=$(curl -s -X  GET "https://raw.githubusercontent.com/cosmos/chain-registry/master/$chain/chain.json" | jq -r '.|.slip44')
  echo "$slip44" || ""
}

# Returns chain_id
function chainid() {
  local chain
  local chain_id

  chain=$(echo "$1" | tr '[:upper:]' '[:lower:]')
  chain_id=$(curl -s -X  GET "https://raw.githubusercontent.com/cosmos/chain-registry/master/$chain/chain.json" | jq -r -e '.|.chain_id')

  echo "$chain_id"
}

function health-check() {
  echo ">>-- Running health-check --<<"
  hermes --config "$CONFIG_LOCATION" health-check
}

function check-balance() {
  hermes --config "$CONFIG_LOCATION" keys balance --chain "$1"
}

# -- Parse arguments ------------------------------------

while getopts "hs:d:r:p:t:" opt; do
  case $opt in
    h)
      usage
      ;;
    s)
      src_chain=$OPTARG
      ;;
    d)
      dst_chain=$OPTARG
      ;;
    r)
      root=$OPTARG
      ;;
    p)
      RPORT=$OPTARG
      ;;
    t)
      TPORT=$OPTARG
      ;;
    \?)
      echo "Invalid option: -$OPTARG" >&2
      usage
      ;;
  esac
done

if [ -z "$src_chain" ]; then
  echo "Source chain not specified"
  usage
fi

if [ -z "$dst_chain" ]; then
  echo "Dest chain file not specified"
  usage
fi

if [ -z "$root" ]; then
  echo "Home folder is not specified, using default: $HOME_DIR"
else
  HOME_DIR=$(readlink -f "$root")
  echo "Home folder is specified: $HOME_DIR"
fi

CONFIG_LOCATION="$HOME_DIR/config.toml"
echo "Config location: $CONFIG_LOCATION"

if [ -z "$RPORT" ]; then
  echo "REST port not specified, using default: $REST_PORT"
else
  REST_PORT=$RPORT
  echo "REST port is specified: $REST_PORT"
fi

if [ -z "$TPORT" ]; then
  echo "TELEMETRY port not specified, using default: $METRIC_PORT"
else
  METRIC_PORT=$TPORT
  echo "TELEMETRY port is specified: $METRIC_PORT"
fi

# -- Body -----------------------------------------------

# Arrays
chains=()
chain_ids=()
bips44=()

echo "Started adding chains to config.toml"
echo "Root folder: $HOME_DIR"
echo "Source chain: $src_chain => Dest chain: $dst_chain"
echo "Rest port: $REST_PORT"
echo "Metric port: $METRIC_PORT"

# Read the file line by line and execute your 1 command for each line
while read -r line; do
  if [[ "$line" == "" ]]; then
    echo "Skipping empty line"
    continue
  fi

  chain=$(echo "$line" | tr '[:upper:]' '[:lower:]')
  chain_id=$(chainid "$chain") || true
  bip44_index=$(bip44_path "$chain") || true

  if [[ -z "$chain_id" || "${chain_id}" == "null" ]]; then
    echo "chain_id not found for $chain"
  else
    echo "- $line ($chain_id)"
    chains+=("$chain:key-$chain_id")
    chain_ids+=("$chain_id")
    bips44+=("$bip44_index")
  fi
done < "$src_chain"

if [ ${#chains[@]} -eq 0 ]; then
    echo "Not found correct chains. Exiting..."
    exit 1
fi

if [ -n "$dst_chain" ] ; then
  auto-config-with-source "$dst_chain" "${chains[@]}" || true
else
  auto-config "${chains[@]}" || true
fi

for i in "${!chain_ids[@]}"; do
  add-key "${chain_ids[$i]}" "${bips44[$i]}" || true
done

enable-all || true
update-max-gas || true
update-ports || true