# Scripts description

# create-channel.sh
===================


This script used to create channels between chains.

### Usage

```
Usage: ./create-channel.sh [-h] [-f file] [-s chain] [-r root]
  -h          Display this help message
  -f file     Specify the file with chains to be configured
  -s chain    Specify the chain which will be used as source for packet filtering
  -r root     Specify config.toml root folder
```

- -f = file with chains list which should be configured as target chains
- -s = chain name which will be used as source chain for packet filtering
- -r = root folder for config.toml files

Status: completed  
Dependencies: none


# create-many-to-many.sh
===================

```
Usage: ./create-channel.sh [-h] [-r root]
  -r root     Specify config.toml root folder
```

This script does the same as `create-channel.sh` but it creates channels between all chains in the source and target files.

This script uses following files to create channels:

- chains.txt = target chains list
- source.txt = source chains list

Usage:
Fill chains.txt and source.txt with chains names and run the script.

Status: completed  
Dependencies: `create-channel.sh`

# create_channels.rb
===================

```shell
./create_channels.rb
```

### Purpose

Created according to what was discussed on the call - create channels between new chains.

### Description

1. It checks if new chains appeared in the github repo
   - Open `chain_chainid.json` and read chains into the memory dictionary
   - Run `update_chain_registry.rb` to update the chains list
   - Find difference between two sets of chains
2. For all new chains it will run `reate-many-to-many.sh`

Status: completed  
Dependencies: `chain_chainid.json`, `update_chain_registry.rb`, `create-many-to-many.sh`

# expired_terra_channels.rb
===================

```shell
./expired_terra_channels.rb
```

This script is used to display expired clients and channels on Terra chain.

Status: completed  
Dependencies: none

# hermes_configure_many.sh
===================

### Purpose

As discussed on the call we must have a way to create multiple hermes instances on the same machine.  
This script will create multiple folders with hermes `config.toml`.  
For each chain pair it will create a folder with name `chain1_chain2` and run `prepare-config.sh`.


Usage:

```
./hermes_configure_many.sh -h
Usage: ./hermes_configure_many.sh [-h] [-s file] [-d file] [-p starting port]
  -h          Display this help message
  -s file     Specify the file with source chains
  -d file     Specify the file with dest chains
  -p file     Specify the starting port from which to search for available ports
```

Status: completed  
Dependencies: `prepare-config.sh`

# install_deps.sh
===================

This script will install ruby gems dependencies.

# prepare-config.sh
===================

This script is used to create `config.toml` files for hermes instances using `-r` as home folder.

```
./prepare-config.sh -h
Usage: ./prepare-config.sh [-h] [-f file] [-s chain] [-r root] [-p port]  [-m port]
  -h          Display this help message
  -f file     Specify the file with chains to be configured
  -s chain    Specify the chain which will be used as source for packet filtering
  -r root     Specify config.toml root folder
  -p port     Specify REST port
  -m port     Specify TELEMETRY port
```

Status: completed  
Dependencies: `$HOME/.hermes/mnemonics/common.txt` = file with mnemonics as source for chains keys

# show_registered.sh
===================

Run this script to display registered chains using TFM endpoints
Status: completed  
Dependencies: none

# hermes_many.rb
===================

```
./hermes_many.rb --help
NAME
    hermes_many.rb - Configure and run hermes instances using channels registry


SYNOPSIS
    hermes_many.rb [global options] command [command options] [arguments...]

GLOBAL OPTIONS
    -h, --home=HOME - The output format (default: /Users/dima/.hermes)
    --help          - Show this message


COMMANDS
    channels     - Show channels used by chain
    config       - Show chain config from registry
    help         - Shows a list of commands or help for one command
    init         - Init multi folder hermes instance using channels registry json files
    init-one     - Init single hermes instance with one config file for one chain. Relaying 1 -> N channels
    proxy-update - Check if proxy installed
    show         - Show running instances
    start        - Start hermes instances
    start-one    - Start single hermes instance with one config file for one chain. Relaying 1 -> N channels
    stop         - Stop running instances
    stop-one     - Stop single hermes instance with one config file for one chain. Relaying 1 -> N channels
```

### - channels

Shows channels created for a chain specified by a flag `-c`

```
./hermes_many.rb channels --help
NAME
    channels - Show channels used by chain

SYNOPSIS

    hermes_many.rb [global options] channels [command options]

COMMAND OPTIONS
    -c, --chain=CHAIN - The source chain (default: none)
```

### - config

Show chain config from GitHub registry

```
./hermes_many.rb config --help
NAME
    config - Show chain config from registry

SYNOPSIS

    hermes_many.rb [global options] config [command options]


COMMAND OPTIONS
    -c, --chain=CHAIN - The chain to show (default: none)
```

### - init

This command uses chain registry files to find existing channels and will create hermes instances for each chain pair.
For example, if there is a channel between chain1 and chain2, and it is registered in the registry file, 
then this command will create a folder with name `chain1_chain2` and will create `config.toml` files for both chains.

This command will traverse all registry files and will create folders for all existing channels.

```
./hermes_many.rb init --help
NAME
    init - Init multi folder hermes instance using channels registry json files

SYNOPSIS

    hermes_many.rb [global options] init [command options]


COMMAND OPTIONS
    -f, --[no-]force - Delete existing folder
```


### - init-one

This command will create a folder with name `chain1_hermes` and will create `config.toml` for all chains  
where chain1 has a channel registered in the registry file.

```
./hermes_many.rb init-one --help
NAME
    init-one - Init single hermes instance with one config file for one chain. Relaying 1 -> N channels

SYNOPSIS

    hermes_many.rb [global options] init-one [command options]


COMMAND OPTIONS
    -c, --chain=CHAIN - Chain for which need to create folder (default: none)
    -f, --[no-]force  - Delete existing folder
```


### - proxy-update

It updates proxies for proxychains4.

```
./hermes_many.rb proxy-update --help
NAME
    proxy-update - Check if proxy installed

SYNOPSIS

    hermes_many.rb [global options] proxy-update
```


### - show

Show running hermes instances created and started by commands `init` and `start`

Every hermes instance created in the folder chainX_chainY will be displayed as running instance.

```
./hermes_many.rb show --help
NAME
    show - Show running instances

SYNOPSIS

    hermes_many.rb [global options] show
```

### - start

Start running hermes instances created and started by commands `init`
Every hermes instance created in the folder chainX_chainY will be started if a flag `-a` is used.
If a flag `-p` is used then a proxy will be used for each hermes instance.

```
./hermes_many.rb start --help
NAME
    start - Start hermes instances

SYNOPSIS

    hermes_many.rb [global options] start [command options]

COMMAND OPTIONS
    -a, --[no-]all   - Run all instances
    -p, --[no-]proxy - Run with proxy
```

### - start-one

Start running hermes instances created and started by commands `init-one`.  
Flag `-c` is used to specify a chain for which a hermes instance will be started.
Folder `chainX_hermes` will be used to find a hermes instance.

```
./hermes_many.rb start-one --help
NAME
    start-one - Start single hermes instance with one config file for one chain. Relaying 1 -> N channels

SYNOPSIS

    hermes_many.rb [global options] start-one [command options]

COMMAND OPTIONS
    -c, --chain=CHAIN - Chain for which need to start instance (default: none)
    -p, --[no-]proxy  - Run with proxy
```


### - stop

Stop running hermes instances created and started by commands `init` and `start`

### - stop-one

Stop running hermes instances created and started by commands `init-one` and `start-one`

### - help

Display help message