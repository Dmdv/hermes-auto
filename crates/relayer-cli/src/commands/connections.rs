use abscissa_core::clap::Parser;
use abscissa_core::{Command, Runnable};
use http::uri::Builder;
use ibc_chain_registry::error::RegistryError;
use ibc_relayer_types::core::ics24_host::identifier::ChainId;
use serde::{Deserialize, Serialize};

const PROTOCOL: &str = "https";
const HOST: &str = "ibc.tfm.com";
const PATH: &str = "/chain/connections?chainId=";
// Example: https://ibc.tfm.com/chain/connections?chainId=phoenix-1

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct ChainInfo {
    pub id: i32,
    #[serde(rename = "chainId")]
    pub chain_id: String,
    #[serde(rename = "chainName")]
    pub chain_name: String,
    #[serde(rename = "prettyName")]
    pub pretty_name: String,
}

#[derive(Clone, Command, Debug, Parser)]
pub struct ConnectionsCmd {
    #[clap(
        long = "chain",
        required = true,
        value_name = "CHAIN_ID",
        help_heading = "FLAGS",
        help = "Identifier of the chain"
    )]
    chain_id: ChainId,
}

impl Runnable for ConnectionsCmd {
    fn run(&self) {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        let res = rt.block_on(async {
            self.get_connected_chains().await
        });

        let list = res.unwrap();

        list.iter().for_each(|conn| {
            println!("{} | {} | {}", conn.chain_id, conn.chain_name, conn.pretty_name);
        });

        println!("Found {} connections to {}:", list.len(), self.chain_id.as_str());
    }
}

impl ConnectionsCmd {
    async fn get_connected_chains(&self) -> Result<Vec<ChainInfo>, RegistryError> {
        let url = Builder::new()
            .scheme(PROTOCOL)
            .authority(HOST)
            .path_and_query(format!("{}{}", PATH, self.chain_id.as_str(),).as_str())
            .build()
            .map_err(|e| RegistryError::url_parse_error(self.chain_id.to_string(), e))?;

        let response = reqwest::get(url.to_string())
            .await
            .map_err(|e| RegistryError::request_error(url.to_string(), e))?;

        if response.status().is_success() {
            match response.text().await {
                Ok(body) => match serde_json::from_str(&body) {
                    Ok(parsed) => Ok(parsed),
                    Err(e) => Err(RegistryError::json_parse_error(self.chain_id.to_string(), e)),
                },
                Err(e) => Err(RegistryError::request_error(url.to_string(), e)),
            }
        } else {
            Err(RegistryError::status_error(
                url.to_string(),
                response.status().as_u16(),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_url() {
        let res = ConnectionsCmd::get_connected_chains(
            &ConnectionsCmd {
                chain_id: ChainId::new("phoenix".to_string(), 1),
            }
        ).await;

        assert!(res.is_ok());

        let list = res.unwrap();
        assert!(list.len() > 0);
    }

    #[test]
    fn connection_list_deserialize() {
        let json = r#"
        [
            {
                "id": 1,
                "chainId": "phoenix-1",
                "chainName": "phoenix",
                "prettyName": "Phoenix",
                "extras": {
                    "fees": {
                        "fee_tokens": [{
                            "denom": "uatom",
                            "low_gas_price": 0.01,
                            "high_gas_price": 0.03,
                            "average_gas_price": 0.025,
                            "fixed_min_gas_price": 0
                        }]
                    },
                    "slip44": 118,
                    "staking": {
                    "staking_tokens": [
                        {
                        "denom": "uatom"
                        }
                    ]
                    },
                        "bech32_prefix": "cosmos"
                }
            },
            {
                "id": 2,
                "chainId": "phoenix-2",
                "chainName": "phoenix",
                "prettyName": "Phoenix"
            }
        ]"#;

        let connections: Vec<ChainInfo> = serde_json::from_str(json).unwrap();
        assert_eq!(connections.len(), 2);
        assert_eq!(connections[0].id, 1);
        assert_eq!(connections[0].chain_id, "phoenix-1");
    }
}
