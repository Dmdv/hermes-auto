# Prerequisites

Clone Hermes repository

```shell
git clone https://github.com/tfm-com/tfm_hermes_relayer.git
```

Build Hermes

```shell
cd hermes && make install
```

Copy scripts to hermes home folder

```shell
mkdir -p $HOME/.hermes && cp -rv ./setup/*.* $HOME/.hermes/
```

Copy registry files to hermes home folder

```shell
cp -rv ./channel_registry/*.* $HOME/.hermes/
```

Install Ruby (Ubuntu)

```shell
sudo apt-get install ruby-full
```

Install dependencies

```shell
cd $HOME/.hermes && ./install_deps.sh
```

Update chain - chain id mapping (chain_chainid.json)

```shell
./update_chain_registry.rb
```

It will update `$HOME/.hermes/chain_chainid.json`


At this stage you will script and registry files ready to use.