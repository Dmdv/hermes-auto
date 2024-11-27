# Hermes configuration

## Hermes' foundation developed commands
Reference: https://hermes.informal.systems/documentation/configuration/configure-hermes.html

## TFM developed commands

## Supporting scripts

### List of files in the Setup folder

- chain_chainid.json    -   the map of chain name to chain id, 
                            used by hermes_many.rb and updated by update_chain_registry.rb
- configure-multiple.sh -   example script how to run multiple hermes instances from channels registry


### hermes_many.rb

#### COMMANDS
- channels - Show channels used by chain
- config   - Show chain config from registry
- help     - Shows a list of commands or help for one command
- init     - Init multi folder hermes instance using channels registry json files
- init-one - Init hermes folder for single hermes instance for many channels using registry json files located in HOME
- show     - Show running instances
- start    - Start hermes instances
- stop     - Stop running instances

### create_channels.rb
Monitors new created channels and creates new channels on the other side.

### update_chain_registry.rb
This script will create a dictionary chain -> chain_id from chain registry

### prepare-config.sh

This script uses the following default variables:

```bash
CONFIG_LOCATION="$HOME/.hermes/config.toml"
MNEMONIC_SOURCE="$HOME/.hermes/mnemonics/common.txt"
```

This script is used to prepare the configuration files for the channels.  
It takes file name with chain name as an argument, source chain name as a filter and does the following.

Example:

This command will do the following:
1. Add each chain from the list to configuration file.
2. Restore keys for each chain from mnemonic file.
3. Enables all endpoints
4. Updates max gas
5. Updates terra2 configuration

```bash
./prepare-config.sh ./chains.txt
```

This script will do the same but will include only packages from terra2 chain.

```bash
./prepare-config.sh ./chains.txt terra2
```

### Health checks

Does multiple RPC calls to the full node, to check for
reachability and some basic APIs are available.

Currently, this checks that:  
    - the node responds OK to `/health` RPC call;  
    - the node has transaction indexing enabled;  
    - the SDK & IBC versions are supported;  

Emits a log warning in case anything is amiss.
Exits early if any health check fails, without doing any
further checks.

### GRPC health check

Does GRPC connection to the full node, to check for reachability and report success or failure.

### Websocket connection

Does Websocket connection to the full node, checks reachability and consensus params, and reports success or failure

### Future tasks

1. Implement private channels
2. Implement route filter between chains

### List of chains with funded wallets

##### Terra2 channels

**Legend:**
- Existed - channel already exists
- Failed - channel creation failed (+ short reason)
- ? - channel is not known

| Chain name      | Channel name | Terra side channel | Existed |  Failed  |                   Wallet Address                   |
|:----------------|:------------:|:------------------:|:-------:|:--------:|:--------------------------------------------------:|
| 8ball           |  channel-18  |    channel-122     |         |          |    8ball1phxc2wg9pcvc9wsqhckmajrgn4qhl7fjuwnxp2    |
| agoric          |  channel-15  |    channel-139     |         |          |   agoric19y24qu84zj38s2qj0agnt0tkmdqsqp7w0ylqay    |
| akash           |  channel-68  |    channel-150     |         |          |    akash1phxc2wg9pcvc9wsqhckmajrgn4qhl7fjeq7vm0    |
| arkh            |  channel-14  |    channel-123     |         |          |    arkh1phxc2wg9pcvc9wsqhckmajrgn4qhl7fjc40hvf     |
| assetmantle     |  channel-20  |    channel-140     |         |          |   mantle1phxc2wg9pcvc9wsqhckmajrgn4qhl7fj2lgwal    |
| axelar          |      -       |         -          |    +    |          |   axelar1phxc2wg9pcvc9wsqhckmajrgn4qhl7fjs49rf5    |
| bandchain       | channel-150  |         ?          |         |          |    band1u9jh5gv4nkeq7ccnje0h2nle6e4df4nuf83j0x     |
| beezee          |      -       |         -          |         | + (grpc) |     bze1phxc2wg9pcvc9wsqhckmajrgn4qhl7fjfllqsz     |
| bitcanna        |  channel-19  |    channel-141     |         |          |    bcna1phxc2wg9pcvc9wsqhckmajrgn4qhl7fjwtr228     |
| bitsong         |  channel-12  |    channel-142     |         |          |   bitsong1gurpmk4f3aez5yrl8sx82wka7rw3dk4t9y3yvw   |
| bostrom         |  channel-16  |    channel-121     |         |          |   bostrom1phxc2wg9pcvc9wsqhckmajrgn4qhl7fjhg8cuj   |
| carbon          |      -       |         -          |    +    |          |    swth1phxc2wg9pcvc9wsqhckmajrgn4qhl7fjt9ea3v     |
| cheqd           |      -       |         -          |         | + (conn) |    cheqd1phxc2wg9pcvc9wsqhckmajrgn4qhl7fj6eltfy    |
| chihuahua       |      -       |         -          |    +    |          |  chihuahua1phxc2wg9pcvc9wsqhckmajrgn4qhl7fjhw79rh  |
| comdex          |      -       |         -          |    +    |          |   comdex1phxc2wg9pcvc9wsqhckmajrgn4qhl7fjn53fmz    |
| cosmoshub       |      -       |         -          |    +    |          |   cosmos1phxc2wg9pcvc9wsqhckmajrgn4qhl7fj5mntz4    |
| crescent        |      -       |         -          |    +    |          |     cre1phxc2wg9pcvc9wsqhckmajrgn4qhl7fjsnqwhc     |
| cryptoorgchain  |  channel-63  |    channel-143     |         |          |     cro13ttgjedk83sxapm0u8fe7e84djqluwc24clvzr     |
| decentr         |      -       |         -          |         | + (grpc) |   decentr1phxc2wg9pcvc9wsqhckmajrgn4qhl7fjl44lra   |
| desmos          |  channel-11  |    channel-144     |         |          |   desmos1x0fsyvm5xtr7wed025jprqn8p0cew3n2c869ej    |
| emoney          |      -       |         -          |         | + (grpc) |   emoney1phxc2wg9pcvc9wsqhckmajrgn4qhl7fjmcfl4g    |
| evmos           |      -       |         -          |         | + (grpc) |    evmos1mtpsyy0vmlftjl6zg03h7vj23262f8vs4sf5vk    |
| fetchhub        |  channel-20  |    channel-145     |         |          |    fetch1phxc2wg9pcvc9wsqhckmajrgn4qhl7fj8x60qz    |
| gravitybridge   |      -       |         -          |    +    |          |   gravity1phxc2wg9pcvc9wsqhckmajrgn4qhl7fjstpn8a   |
| injective       |      -       |         -          |    +    |          |     inj1mtpsyy0vmlftjl6zg03h7vj23262f8vsac07yx     |
| IRISnet         |  channel-41  |    channel-146     |         |          |     iaa1phxc2wg9pcvc9wsqhckmajrgn4qhl7fjpen6qy     |
| impacthub       |      -       |         -          |         | + (gas)  |     ixo1phxc2wg9pcvc9wsqhckmajrgn4qhl7fjtwdexx     |
| Jackal          |  channel-4   |    channel-124     |         |          |     jkl1phxc2wg9pcvc9wsqhckmajrgn4qhl7fjd9a6m2     |
| Juno            |      -       |         -          |    +    |          |    juno1phxc2wg9pcvc9wsqhckmajrgn4qhl7fjzfss9f     |
| Kava            |  channel-89  |    channel-147     |         |          |    kava1ef26vfahtlccpemyuts6fg2vzfps0a2qtt0hgt     |
| Kichain         |  channel-22  |    channel-137     |         |          |     ki1phxc2wg9pcvc9wsqhckmajrgn4qhl7fj9kzyxp      |
| Kujira          |      -       |         -          |    +    |          |   kujira1phxc2wg9pcvc9wsqhckmajrgn4qhl7fj9n3n0l    |
| LikeCoin        |  channel-13  |    channel-120     |         |          |    like1phxc2wg9pcvc9wsqhckmajrgn4qhl7fj880fpw     |
| mars            |      -       |         -          |    +    |          |    mars1phxc2wg9pcvc9wsqhckmajrgn4qhl7fjfx2jhw     |
| migaloo         |      -       |         -          |    +    |          |   migaloo1phxc2wg9pcvc9wsqhckmajrgn4qhl7fje063hm   |
| Odin            |  channel-27  |    channel-126     |         |          |    odin1phxc2wg9pcvc9wsqhckmajrgn4qhl7fjsm4gwl     |
| Oraichain       |      -       |         -          |         | + (grpc) |    orai1phxc2wg9pcvc9wsqhckmajrgn4qhl7fj8g9grx     |
| Osmosis         |      -       |         -          |    +    |          |    osmo1phxc2wg9pcvc9wsqhckmajrgn4qhl7fjuqqm58     |
| Persistence     |  channel-85  |    channel-127     |         |          | persistence1phxc2wg9pcvc9wsqhckmajrgn4qhl7fj6h4cv3 |
| Planq           |      -       |         -          |         | + (grpc) |     plq1mtpsyy0vmlftjl6zg03h7vj23262f8vs7fgxsd     |
| Quasar          |      -       |         -          |         | + (gas)  |   quasar1phxc2wg9pcvc9wsqhckmajrgn4qhl7fj6cfk0s    |
| Quicksilver     |  channel-56  |    channel-128     |         |          |    quick1phxc2wg9pcvc9wsqhckmajrgn4qhl7fjllrem8    |
| Regen           | channel-104  |    channel-129     |         |          |    regen1phxc2wg9pcvc9wsqhckmajrgn4qhl7fjtech53    |
| secretnetwork   |      -       |         -          |    +    |          |   secret1xfx5nlck64yatxq8km07tjs7rfwjyq2r8jp3jn    |
| Sentinel        |  channel-72  |    channel-130     |         |          |    sent1phxc2wg9pcvc9wsqhckmajrgn4qhl7fj0q9jx6     |
| Sommelier       |  channel-7   |    channel-131     |         |          |    somm1phxc2wg9pcvc9wsqhckmajrgn4qhl7fjc8u8nl     |
| Stargaze        | channel-148  |    channel-132     |         |          |    stars1phxc2wg9pcvc9wsqhckmajrgn4qhl7fjq8ykfy    |
| Starname        |  channel-27  |    channel-133     |         |          |    star1cu2lztjl3r7mj98tag52kywl5nqvl73c0njenx     |
| Stride          |      -       |         -          |    +    |          |   stride1phxc2wg9pcvc9wsqhckmajrgn4qhl7fjhsnhke    |
| Teritori        |  channel-15  |    channel-138     |         |          |    tori1phxc2wg9pcvc9wsqhckmajrgn4qhl7fjk0yze9     |
| Terra           |  channel-73  |    channel-134     |         |          |    terra1p8378nuqgmxaprlgtk5y7tr76g6t4n6mhfk7qt    |
| Tgrade          |      -       |         -          |         | + (grpc) |   tgrade1phxc2wg9pcvc9wsqhckmajrgn4qhl7fjlmm4v9    |
| Umee            |  channel-54  |    channel-135     |         |          |    umee1phxc2wg9pcvc9wsqhckmajrgn4qhl7fjxdw5x8     |
| Vidulum         |  channel-14  |    channel-136     |         |          |     vdl13s3u8cn8ncq3lqpjcp2q7s9gjx06mghep907dq     |
| Terra2 (source) |      -       |         -          |    -    |    -     |    terra1p8378nuqgmxaprlgtk5y7tr76g6t4n6mhfk7qt    |
