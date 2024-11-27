pub const REGISTRY_PATH: &str = "/osmosis-labs/assetlists";
pub const BRANCH: &str = "main";
pub const OSMOSIS_PATH: &str = "osmosis-1/osmosis-1.assetlist.json";

use abscissa_core::clap::Parser;
use abscissa_core::{Command, Runnable};
use ibc_chain_registry::asset_list::{AssetList};
use ibc_chain_registry::fetchable_github::FetchableGitHub;

#[derive(Clone, Command, Debug, Parser)]
pub struct OsmosisTokensCmd {}

impl Runnable for OsmosisTokensCmd {
    fn run(&self) {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        let res = rt.block_on(async {
            AssetList::fetch_from_path(
                OSMOSIS_PATH.to_string(),
                REGISTRY_PATH.to_string(),
                BRANCH.to_string(),
                None,
            ).await
        });

        let list = res.unwrap();

        println!("{} tokens available in Osmosis:", list.assets.len());

        list.assets.iter().for_each(|asset| {
            println!("{}: {}", asset.symbol, asset.name);
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn asset_fetch_test() {
        // https://raw.githubusercontent.com/osmosis-labs/assetlists/main/osmosis-1/osmosis-1.assetlist.json

        let th = tokio::spawn(AssetList::fetch_from_path(
            OSMOSIS_PATH.to_string(),
            REGISTRY_PATH.to_string(),
            BRANCH.to_string(),
            None,
        )).await;

        let res = th.unwrap();

        assert!(res.is_ok());

        let list = res.unwrap();
        assert!(list.assets.len() > 0);
    }
}
