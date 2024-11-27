#!/bin/bash
set -e

target='./chains.txt'
source='./source.txt'

err_report() {
    echo "Error on line $1"
}

trap 'err_report $LINENO' ERR

function usage {
  echo "Usage: $0 [-h] [-r root]"
  echo "  -h          Display this help message"
  echo "  -r root     Specify config.toml root folder"
  exit 1
}

# -- Parse arguments ------------------------------------

while getopts "hr:" opt; do
  case $opt in
    h)
      usage
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

if [ -z "$root" ]; then
  echo "Root folder not specified"
  usage
fi

echo "Using root folder: $root"


while read -r line; do
  if [[ "$line" == "" ]]; then
    echo "Skipping empty line"
    continue
  fi

  chain=$(echo "$line" | tr '[:upper:]' '[:lower:]')

  echo "---"
  echo ">> Started creating channels from $chain to all"

  ./create-channel.sh -f "$target" -s "$chain" -r "$root" || true

done < "$source"