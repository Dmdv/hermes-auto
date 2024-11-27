#!/bin/bash
set -e

./daemon_proxy_config.rb start
./daemon_create_channels.rb start
./daemon_update_chain_registry.rb start
