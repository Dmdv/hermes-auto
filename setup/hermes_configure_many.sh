#!/bin/bash
set -e

err_report() {
    echo "Error on line $1"
}

trap 'err_report $LINENO' ERR

function usage {
  echo "Usage: $0 [-h] [-s file] [-d file] [-p starting port]"
  echo "  -h          Display this help message"
  echo "  -s file     Specify the file with source chains"
  echo "  -d file     Specify the file with dest chains"
  echo "  -p file     Specify the starting port from which to search for available ports"
  exit 1
}

while getopts "hs:d:p:" opt; do
  case $opt in
    h)
      usage
      ;;
    s)
      source_file=$OPTARG
      ;;
    d)
      dest_file=$OPTARG
      ;;
    p)
      start_port=$OPTARG
      ;;
    \?)
      echo "Invalid option: -$OPTARG" >&2
      usage
      ;;
  esac
done

if [ -z "$start_port" ]; then
  start_port=3000
  echo "Starting port is not specified, using default: $start_port"
fi

if [ -z "$source_file" ]; then
  echo "Source chains file not specified"
  usage
fi

if [ -z "$dest_file" ]; then
  echo "Dest chains file not specified"
  usage
fi

if [ ! -f "$source_file" ]; then
  echo "File not found: $source_file"
  exit 1
fi

if [ ! -f "$dest_file" ]; then
  echo "File not found: $dest_file"
  exit 1
fi

LAST_RPORT=$start_port
LAST_TPORT=$((LAST_RPORT+1))

while read -r src; do
  while read -r dest; do

    if [[ "$src" == "$dest" ]]; then
        echo "Skipping source chain"
        continue
    fi

    folder="${src}_${dest}"
    echo "----------------------------------------"
    echo "Creating for ${src} => ${dest}"
    echo "----------------------------------------"
    mkdir -p "$folder"
    rm -rf "$folder"/source.txt

    {
      echo "$src"
      echo "$dest"
    } >> "$folder"/source.txt

    P=$LAST_RPORT
    T=$LAST_TPORT

    while true; do
      echo "Checking port $LAST_RPORT"
      if ! nc -z localhost $LAST_RPORT >/dev/null; then
        echo "Port $LAST_RPORT is available"
        P=$LAST_RPORT
        LAST_RPORT=$((LAST_RPORT+1))
        break
      else
        echo "Port $LAST_RPORT is not available"
        LAST_RPORT=$((LAST_RPORT+1))
      fi
    done

    LAST_TPORT=$LAST_RPORT

    while true; do
      echo "Checking port $LAST_TPORT"
      if ! nc -z localhost $LAST_TPORT >/dev/null; then
        echo "Port $LAST_TPORT is available"
        T=$LAST_TPORT
        LAST_TPORT=$((LAST_TPORT+1))
        break
      else
        echo "Port $LAST_TPORT is not available"
        LAST_TPORT=$((LAST_TPORT+1))
      fi
    done

    LAST_RPORT=$LAST_TPORT

    echo "Using port $T for TELEMETRY"
    echo "Using port $P for REST"

    ./prepare-config.sh -f "$folder"/source.txt -s "${src}" -r "$folder" -p "$P" -m "$T"

#    echo "----------------------------------------"
#    echo "         Starting endpoints updating"
#    echo "----------------------------------------"
#    hermes --config "$folder/config.toml" config endpoints

  done < "$dest_file"
done < "$source_file"
