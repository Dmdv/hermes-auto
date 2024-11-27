# Use case - New chains with missing channels

This workflow will show how to configure Hermes for 2 chains with missing channels.

[Prerequisites](./PREREQUISITES.MD)

1. Make sure chains exist in https://github.com/cosmos/chain-registry
2. CD to the hermes home folder `$HOME/.hermes`

## Configure Hermes configuration for 2 chains

Take as example 2 chains:
- noble (https://github.com/cosmos/chain-registry/tree/master/noble)
- noise (https://github.com/cosmos/chain-registry/tree/master/nois)

### Prepare arguments

```shell
cd $HOME/.hermes/
rm -rf ./chains.txt
echo "noble" >> ./chains.txt
echo "nois" >> ./chains.txt
```

It will create a file `chains.txt` with the list of configurable chains


### Run autoconfiguration

Using file created on the previous step, run the following command:

```shell
./prepare-config.sh -f chains.txt
```

Sample output with successful result (arkh and axelar chains):

```
./prepare-config.sh -f chains.txt
Source chain is not specified
Using auto-config without source chain
Home folder is not specified, using default: /Users/dima/.hermes
Config location: /Users/dima/.hermes/config.toml
REST port not specified, using default: 3000
TELEMETRY port not specified, using default: 3001
Started adding chains to config.toml
Root folder: /Users/dima/.hermes
Source file: chains.txt
Source chain:
Rest port: 3000
Metric port: 3001
-------------------------------------------------------------------------
** Adding to arkh configuration
** Chain: arkh
** Chain ID: arkh
** Bip44 Index: 118
-------------------------------------------------------------------------
-------------------------------------------------------------------------
** Adding to axelar configuration
** Chain: axelar
** Chain ID: axelar-dojo-1
** Bip44 Index: 118
-------------------------------------------------------------------------
*****************************************************************
       Running auto-config
*****************************************************************
Config location: /Users/dima/.hermes/config.toml
Adding chains to config file: arkh:key-arkh axelar:key-axelar-dojo-1
2023-06-26T21:40:23.411517Z  INFO ThreadId(01) using default configuration from '/Users/dima/.hermes/config.toml'
2023-06-26T21:40:23.414240Z  INFO ThreadId(01) running Hermes v1.5.1+3402b221-dirty
2023-06-26T21:40:23.415644Z  INFO ThreadId(01) Fetching configuration for chains: ["arkh", "axelar"]
2023-06-26T21:40:23.415699Z  INFO ThreadId(01) Using HOME directory from config file: "/Users/dima/.hermes"
2023-06-26T21:40:23.415765Z  INFO ThreadId(01) Getting configuration...
2023-06-26T21:40:23.415768Z  INFO ThreadId(01) Chains: '["arkh", "axelar"]'
2023-06-26T21:40:23.415770Z  INFO ThreadId(01) Source chain: ''
2023-06-26T21:40:25.078900Z  INFO ThreadId(01) Adding packet filters from TFM endpoint without source chain
2023-06-26T21:40:25.078954Z  INFO ThreadId(01) Building paths for arkh -> axelar
2023-06-26T21:40:25.079047Z  INFO ThreadId(01) Looking for arkh -> axelar (arkh -> axelar-dojo-1)
2023-06-26T21:40:25.079110Z  INFO ThreadId(01) Querying TFM registry at https://ibc.tfm.com/channels/pairs?sourceChainId=arkh&destinationChainId=axelar-dojo-1&page=1&take=1000
2023-06-26T21:40:26.048147Z  INFO ThreadId(01) Getting HOME from config path: "/Users/dima/.hermes/config.toml"
2023-06-26T21:40:26.048458Z  INFO ThreadId(01) Channel config file HOME dir: "/Users/dima/.hermes"
2023-06-26T21:40:26.048482Z  INFO ThreadId(01) Channel config file: "/Users/dima/.hermes/arkh-channels.json"
2023-06-26T21:40:26.048569Z  INFO ThreadId(01) Trying to load JSON in load_json:"/Users/dima/.hermes/arkh-channels.json"
2023-06-26T21:40:26.051126Z  INFO ThreadId(01) Query channels from file registry: "/Users/dima/.hermes/arkh-channels.json"
2023-06-26T21:40:26.051229Z  INFO ThreadId(01) Found channels for arkh -> axelar-dojo-1 = [ channel-23 -> channel-81 ]
2023-06-26T21:40:26.051364Z  INFO ThreadId(01) Creating config for chain arkh
2023-06-26T21:40:26.051811Z  INFO ThreadId(01) Starting RPC queries for chain: arkh
2023-06-26T21:40:26.051821Z  INFO ThreadId(01) Querying chain `arkh` with 5 retries
2023-06-26T21:40:28.156997Z  INFO ThreadId(01) Query of arkh succeeded with response HermesConfigData { rpc_address: Url { inner: Url { scheme: "https", cannot_be_a_base: false, username: "", password: None, host: Some(Domain("rpc.arkh.nodestake.top")), port: None, path: "/", query: None, fragment: None }, scheme: Https, host: "rpc.arkh.nodestake.top", port: 443 }, max_block_size: 22020096, websocket: Url { inner: Url { scheme: "wss", cannot_be_a_base: false, username: "", password: None, host: Some(Domain("rpc.arkh.nodestake.top")), port: None, path: "/websocket", query: None, fragment: None }, scheme: SecureWebSocket, host: "rpc.arkh.nodestake.top", port: 443 } }
2023-06-26T21:40:28.157128Z  INFO ThreadId(01) Starting gRPC queries for chain: arkh
2023-06-26T21:40:28.157147Z  INFO ThreadId(01) Querying chain `arkh` with 5 retries
2023-06-26T21:40:29.156250Z  INFO ThreadId(01) Query of arkh succeeded with response Url { inner: Url { scheme: "https", cannot_be_a_base: false, username: "", password: None, host: Some(Domain("grpc.arkh.nodestake.top")), port: None, path: "/", query: None, fragment: None }, scheme: Https, host: "grpc.arkh.nodestake.top", port: 443 }
2023-06-26T21:40:29.156329Z  INFO ThreadId(01) Chain: arkh, found first healthy gRPC: https://grpc.arkh.nodestake.top/
2023-06-26T21:40:29.157626Z  INFO ThreadId(01) Creating config for chain axelar
2023-06-26T21:40:29.157689Z  INFO ThreadId(01) Starting RPC queries for chain: axelar
2023-06-26T21:40:29.157699Z  INFO ThreadId(01) Querying chain `axelar` with 5 retries
2023-06-26T21:40:31.967355Z  INFO ThreadId(01) Query of axelar succeeded with response HermesConfigData { rpc_address: Url { inner: Url { scheme: "https", cannot_be_a_base: false, username: "", password: None, host: Some(Domain("axelar-rpc.rockrpc.net")), port: None, path: "/", query: None, fragment: None }, scheme: Https, host: "axelar-rpc.rockrpc.net", port: 443 }, max_block_size: 22020096, websocket: Url { inner: Url { scheme: "wss", cannot_be_a_base: false, username: "", password: None, host: Some(Domain("axelar-rpc.rockrpc.net")), port: None, path: "/websocket", query: None, fragment: None }, scheme: SecureWebSocket, host: "axelar-rpc.rockrpc.net", port: 443 } }
2023-06-26T21:40:31.967459Z  INFO ThreadId(01) Starting gRPC queries for chain: axelar
2023-06-26T21:40:31.967478Z  INFO ThreadId(01) Querying chain `axelar` with 5 retries
2023-06-26T21:40:33.333351Z  INFO ThreadId(01) Query of axelar succeeded with response Url { inner: Url { scheme: "https", cannot_be_a_base: false, username: "", password: None, host: Some(Domain("axelar-mainnet-grpc.autostake.com")), port: None, path: "/", query: None, fragment: None }, scheme: Https, host: "axelar-mainnet-grpc.autostake.com", port: 443 }
2023-06-26T21:40:33.333418Z  INFO ThreadId(01) Chain: axelar, found first healthy gRPC: https://axelar-mainnet-grpc.autostake.com/
2023-06-26T21:40:33.333502Z  INFO ThreadId(01) arkh: uses key "key-arkh"
2023-06-26T21:40:33.333511Z  INFO ThreadId(01) axelar-dojo-1: uses key "key-axelar-dojo-1"
2023-06-26T21:40:33.337243Z  WARN ThreadId(01) Gas parameters are set to default values.
SUCCESS "Config file written successfully : /Users/dima/.hermes/config.toml."
Adding key for chain_id arkh
2023-06-26T21:40:33.357606Z  INFO ThreadId(01) running Hermes v1.5.1+3402b221-dirty
2023-06-26T21:40:33.357634Z  INFO ThreadId(01) Getting HOME from config path: "/Users/dima/.hermes/config.toml"
2023-06-26T21:40:33.357662Z  INFO ThreadId(01) Home directory: "/Users/dima/.hermes"
SUCCESS Restored key 'key-arkh' (arkh1phxc2wg9pcvc9wsqhckmajrgn4qhl7fjc40hvf) on chain arkh
Adding key for chain_id axelar-dojo-1
2023-06-26T21:40:33.379746Z  INFO ThreadId(01) running Hermes v1.5.1+3402b221-dirty
2023-06-26T21:40:33.379773Z  INFO ThreadId(01) Getting HOME from config path: "/Users/dima/.hermes/config.toml"
2023-06-26T21:40:33.379793Z  INFO ThreadId(01) Home directory: "/Users/dima/.hermes"
SUCCESS Restored key 'key-axelar-dojo-1' (axelar1phxc2wg9pcvc9wsqhckmajrgn4qhl7fjs49rf5) on chain axelar-dojo-1
```

Sample output with UNSUCCESSFUL result (nois and noble chains):
Example when some of the chain has no healthy gRPC endpoints:

```
./prepare-config.sh -f chains.txt
Source chain is not specified
Using auto-config without source chain
Home folder is not specified, using default: /Users/dima/.hermes
Config location: /Users/dima/.hermes/config.toml
REST port not specified, using default: 3000
TELEMETRY port not specified, using default: 3001
Started adding chains to config.toml
Root folder: /Users/dima/.hermes
Source file: chains.txt
Source chain:
Rest port: 3000
Metric port: 3001
-------------------------------------------------------------------------
** Adding to noble configuration
** Chain: noble
** Chain ID: noble-1
** Bip44 Index: 118
-------------------------------------------------------------------------
-------------------------------------------------------------------------
** Adding to nois configuration
** Chain: nois
** Chain ID: nois-1
** Bip44 Index: 118
-------------------------------------------------------------------------
*****************************************************************
       Running auto-config
*****************************************************************
Config location: /Users/dima/.hermes/config.toml
Adding chains to config file: noble:key-noble-1 nois:key-nois-1
2023-06-26T21:39:37.366312Z  INFO ThreadId(01) using default configuration from '/Users/dima/.hermes/config.toml'
2023-06-26T21:39:37.369060Z  INFO ThreadId(01) running Hermes v1.5.1+3402b221-dirty
2023-06-26T21:39:37.370833Z  INFO ThreadId(01) Fetching configuration for chains: ["noble", "nois"]
2023-06-26T21:39:37.370892Z  INFO ThreadId(01) Using HOME directory from config file: "/Users/dima/.hermes"
2023-06-26T21:39:37.370962Z  INFO ThreadId(01) Getting configuration...
2023-06-26T21:39:37.370965Z  INFO ThreadId(01) Chains: '["noble", "nois"]'
2023-06-26T21:39:37.370968Z  INFO ThreadId(01) Source chain: ''
2023-06-26T21:39:39.098160Z  INFO ThreadId(01) Adding packet filters from TFM endpoint without source chain
2023-06-26T21:39:39.098214Z  INFO ThreadId(01) Building paths for noble -> nois
2023-06-26T21:39:39.098230Z  INFO ThreadId(01) Looking for noble -> nois (noble-1 -> nois-1)
2023-06-26T21:39:39.098294Z  INFO ThreadId(01) Querying TFM registry at https://ibc.tfm.com/channels/pairs?sourceChainId=noble-1&destinationChainId=nois-1&page=1&take=1000
2023-06-26T21:39:40.215536Z  INFO ThreadId(01) Getting HOME from config path: "/Users/dima/.hermes/config.toml"
2023-06-26T21:39:40.215832Z  INFO ThreadId(01) Channel config file HOME dir: "/Users/dima/.hermes"
2023-06-26T21:39:40.215852Z  INFO ThreadId(01) Channel config file: "/Users/dima/.hermes/noble-1-channels.json"
2023-06-26T21:39:40.217046Z  INFO ThreadId(01) Trying to load JSON in load_json:"/Users/dima/.hermes/noble-1-channels.json"
2023-06-26T21:39:40.218392Z  INFO ThreadId(01) Query channels from file registry: "/Users/dima/.hermes/noble-1-channels.json"
2023-06-26T21:39:40.218421Z  WARN ThreadId(01) Error loading channel config file: config I/O error: No such file or directory (os error 2)
2023-06-26T21:39:40.219853Z  INFO ThreadId(01) Creating config for chain noble
2023-06-26T21:39:40.220550Z  INFO ThreadId(01) Starting RPC queries for chain: noble
2023-06-26T21:39:40.220569Z  INFO ThreadId(01) Querying chain `noble` with 5 retries
2023-06-26T21:39:41.300846Z  INFO ThreadId(01) Retry 0 failed to query noble endpoints
2023-06-26T21:39:42.243129Z  INFO ThreadId(01) Retry 1 failed to query noble endpoints
2023-06-26T21:39:43.153088Z  INFO ThreadId(01) Retry 2 failed to query noble endpoints
2023-06-26T21:39:44.047187Z  INFO ThreadId(01) Retry 3 failed to query noble endpoints
2023-06-26T21:39:44.945276Z  INFO ThreadId(01) Retry 4 failed to query noble endpoints
2023-06-26T21:39:44.945377Z  INFO ThreadId(01) Creating config for chain nois
2023-06-26T21:39:44.945425Z  INFO ThreadId(01) Starting RPC queries for chain: nois
2023-06-26T21:39:44.945436Z  INFO ThreadId(01) Querying chain `nois` with 5 retries
2023-06-26T21:39:47.600208Z  INFO ThreadId(01) Query of nois succeeded with response HermesConfigData { rpc_address: Url { inner: Url { scheme: "https", cannot_be_a_base: false, username: "", password: None, host: Some(Domain("nois.kingnodes.com")), port: None, path: "/", query: None, fragment: None }, scheme: Https, host: "nois.kingnodes.com", port: 443 }, max_block_size: 22020096, websocket: Url { inner: Url { scheme: "wss", cannot_be_a_base: false, username: "", password: None, host: Some(Domain("nois.kingnodes.com")), port: None, path: "/websocket", query: None, fragment: None }, scheme: SecureWebSocket, host: "nois.kingnodes.com", port: 443 } }
2023-06-26T21:39:47.600854Z  INFO ThreadId(01) Starting gRPC queries for chain: nois
2023-06-26T21:39:47.600873Z  INFO ThreadId(01) Querying chain `nois` with 5 retries
2023-06-26T21:39:49.383761Z  INFO ThreadId(01) Query of nois succeeded with response Url { inner: Url { scheme: "http", cannot_be_a_base: false, username: "", password: None, host: Some(Domain("nois.grpc.m.stavr.tech")), port: Some(191), path: "/", query: None, fragment: None }, scheme: Http, host: "nois.grpc.m.stavr.tech", port: 191 }
2023-06-26T21:39:49.383796Z  INFO ThreadId(01) Chain: nois, found first healthy gRPC: http://nois.grpc.m.stavr.tech:191/
2023-06-26T21:39:49.384272Z ERROR ThreadId(01) Failed to fetch chain configs for one or every chains in AutoCmd
2023-06-26T21:39:49.384290Z ERROR ThreadId(01) Error finding a healthy endpoint for `noble` after 5 retries. Endpoints: https://noble-rpc.polkachu.com
ERROR HINT: Try to find healthy endpoint for the specified chain or update it's configuration in github
Adding key for chain_id noble-1
2023-06-26T21:39:49.400578Z  INFO ThreadId(01) running Hermes v1.5.1+3402b221-dirty
2023-06-26T21:39:49.400608Z  INFO ThreadId(01) Getting HOME from config path: "/Users/dima/.hermes/config.toml"
2023-06-26T21:39:49.400635Z  INFO ThreadId(01) Home directory: "/Users/dima/.hermes"
ERROR chain 'noble-1' not found in configuration file
Adding key for chain_id nois-1
2023-06-26T21:39:49.411891Z  INFO ThreadId(01) running Hermes v1.5.1+3402b221-dirty
2023-06-26T21:39:49.411915Z  INFO ThreadId(01) Getting HOME from config path: "/Users/dima/.hermes/config.toml"
2023-06-26T21:39:49.411938Z  INFO ThreadId(01) Home directory: "/Users/dima/.hermes"
ERROR chain 'nois-1' not found in configuration file
```

## Update endpoints

This command will check all endpoints from chain registry in GitHub and update them if needed 
(in case they are not healthy in the config file)

```shell
hermes --config $HOME/.hermes/config.toml config endpoints
```

## Create channels

- `-f` is the source file with chains to create channels for
- `-s` is  the source chain to create channels from. It must be present in the source file `chains.txt` in this example

```shell
./create-channel.sh -f chains.txt -s arkh
```
This command will create channels from `arkh` to all other chains in the file `chains.txt`