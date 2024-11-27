use crate::commands::connections::ChainInfo;
use abscissa_core::clap::Parser;
use abscissa_core::{Command, Runnable};
use ibc_chain_registry::error::RegistryError;

const PATH: &str = "https://ibc.tfm.com/chain?network=mainnet";

// Example: https://ibc.tfm.com/chain?network=mainnet

#[derive(Clone, Command, Debug, Parser)]
pub struct ChainsEnumerableCmd {}

impl Runnable for ChainsEnumerableCmd {
    fn run(&self) {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        let res = rt.block_on(async {
            self.get_chains().await
        });

        let list = res.unwrap();

        list.iter().for_each(|conn| {
            println!(
                "{} | {} | {}",
                conn.chain_id, conn.chain_name, conn.pretty_name
            );
        });
    }
}

impl ChainsEnumerableCmd {
    async fn get_chains(&self) -> Result<Vec<ChainInfo>, RegistryError> {
        let response = reqwest::get(PATH.to_string())
            .await
            .map_err(|e| RegistryError::request_error(PATH.to_string(), e))?;

        if response.status().is_success() {
            let list: Vec<ChainInfo> = response
                .json()
                .await
                .map_err(|e| RegistryError::request_error(PATH.to_string(), e))?;

            Ok(list)
        } else {
            Err(RegistryError::status_error(
                PATH.to_string(),
                response.status().as_u16(),
            ))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_url() {
        let res = ChainsEnumerableCmd::get_chains(&ChainsEnumerableCmd {}).await;

        assert_eq!(res.is_ok(), true);
        let list = res.unwrap();
        assert!(list.len() > 0);
    }
}
