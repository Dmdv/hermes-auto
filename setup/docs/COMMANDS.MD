# Hermes commands

Here will be listed commands which have practical application without using scripts.

## Create config file for chains

```shell
 hermes config auto --output ~/.hermes/config.toml --chains chainX:key-chainX chainY:key-chainY
```

## Auto updated chain endpoints

This command will create config file for chains and save it to `~/.hermes/config.toml`

```shell
hermes --config $HOME/.hermes/config.toml config endpoints
```

## Health checks

```shell
hermes --config $HOME/.hermes  health-check
```

## Query Channels

When channel exists

```shell
hermes channels query --source umee-1 --dest juno-1
```

```
hermes channels query --source umee-1 --dest juno-1
2023-06-23T15:38:09.373437Z  INFO ThreadId(01) using default configuration from '/Users/dima/.hermes/config.toml'
2023-06-23T15:38:09.373921Z  INFO ThreadId(01) running Hermes v1.5.1+d170f8cb-dirty
2023-06-23T15:38:09.374221Z  INFO ThreadId(01) Querying TFM registry at https://ibc.tfm.com/channels/pairs?sourceChainId=umee-1&destinationChainId=juno-1&page=1&take=1000
2023-06-23T15:38:10.711045Z  INFO ThreadId(01) Getting HOME from config path: "/Users/dima/.hermes/config.toml"
2023-06-23T15:38:10.711246Z  INFO ThreadId(01) Channel config file HOME dir: "/Users/dima/.hermes"
2023-06-23T15:38:10.711265Z  INFO ThreadId(01) Channel config file: "/Users/dima/.hermes/umee-1-channels.json"
2023-06-23T15:38:10.711313Z  INFO ThreadId(01) Trying to load JSON in load_json:"/Users/dima/.hermes/umee-1-channels.json"
2023-06-23T15:38:10.714103Z  INFO ThreadId(01) Query channels from file registry: "/Users/dima/.hermes/umee-1-channels.json"
channel-43 - channel-208 (UMEE)
channel-43 - channel-208 (JUNO)
SUCCESS "Channels exist"
```

When channel exists but registry file is missing

```shell
hermes channels query --source umee-1 --dest juno-1
```

## Create new channel

```shell
hermes channels create --source chainX --dest chainY --a-port transfer --b-port transfer
```
