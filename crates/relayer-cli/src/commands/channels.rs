use http::uri::Builder;
use serde::{Deserialize, Serialize};
use crate::prelude::*;
use crate::cli_utils::{ChainHandlePair};
use crate::conclude::{exit_with_unrecoverable_error, Output};
use dirs_next::home_dir;

use abscissa_core::{
    clap::Parser,
    Command,
    Runnable
};
use ibc_relayer::{
    foreign_client::ForeignClient,
    connection::Connection,
    channel::Channel,
    config::{
        store_json,
        load_json,
        default::connection_delay
    },
};
use ibc_relayer_types::{
    core::{
        ics04_channel::{
            channel::Ordering,
            version::Version
        },
        ics24_host::identifier::{ChainId, PortId}
    }
};
use ibc_chain_registry::{
    error::RegistryError,
    paths::{
        ChannelConfig,
        ChannelPathInfo,
        IBCPathInfo,
    },
};
use std::fs::File;
use std::path::PathBuf;
use ibc_chain_registry::paths::ChainShortInfo;

const PROTOCOL: &str = "https";
const HOST: &str = "ibc.tfm.com";
// Example: https://ibc.tfm.com/channels/pairs?sourceChainId=phoenix-1&destinationChainId=osmosis-1&page=1&take=1000

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct BaseDenomInfo {
    pub id: i32,
    #[serde(rename = "chainId")]
    chain_id: i32,
    pub base: String,
    pub display: String,
    pub name: String,
    pub symbol: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct ChannelInfo {
    #[serde(rename = "sourceChainId")]
    pub source_chain_id: String,
    #[serde(rename = "destinationChainId")]
    pub destination_chain_id: String,
    #[serde(rename = "channelId")]
    pub source_channel_id: String,
    #[serde(rename = "counterpartyChannelId")]
    pub destination_channel_id: String,
    #[serde(rename = "portId")]
    pub port_id: String,
    #[serde(rename = "counterpartyPortId")]
    pub destination_port_id: String,
    #[serde(rename = "sourceDenom")]
    pub source_denom: String,
    #[serde(rename = "destinationDenom")]
    pub destination_denom: String,
    #[serde(rename = "baseDenom")]
    pub base_denom: BaseDenomInfo,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Channels {
    pub data: Vec<ChannelInfo>,
}

#[derive(Command, Debug, Parser, Runnable)]
pub enum ChannelsCmd {
    #[clap(
        name = "query",
        about = "Query channels using IBC or REST API",
        help = "Query channels using IBC with ibc flag or REST API without ibc flag"
    )]
    Query(ChannelsQueryCmd),
    #[clap(
        name = "create",
        about = "Create channels if there are no channels between the two chains",
        help = "Create channels if there are no channels between the two chains"
    )]
    Create(ChannelsCreateCmd),
    #[clap(
        name = "update-config",
        about = "Auto update config channel filter using information about channels",
        help = "Auto update config channel filter using information about channels"
    )]
    UpdateConfig(ChannelUpdateToConfigCmd),
}

#[derive(Clone, Command, Debug, Parser)]
pub struct ChannelUpdateToConfigCmd {
    #[clap(
        long = "source",
        required = true,
        value_name = "SOURCE_CHAIN_ID",
        help_heading = "REQUIRED",
        help = "Identifier of the source chain"
    )]
    source_chain_id: ChainId,
    #[clap(
        long = "dest",
        required = true,
        value_name = "DESTINATION_CHAIN_ID",
        help_heading = "REQUIRED",
        help = "Identifier of the destination chain"
    )]
    destination_chain_id: ChainId,
}

/// Channels should be created in case there are no existing channels between the two chains
#[derive(Clone, Command, Debug, Parser)]
pub struct ChannelsCreateCmd {
    #[clap(
        long = "source",
        required = true,
        value_name = "SOURCE_CHAIN_ID",
        help_heading = "REQUIRED",
        help = "Identifier of the source chain"
    )]
    source_chain_id: ChainId,

    #[clap(
        long = "dest",
        required = true,
        value_name = "DESTINATION_CHAIN_ID",
        help_heading = "REQUIRED",
        help = "Identifier of the destination chain"
    )]
    destination_chain_id: ChainId,

    #[clap(
        long = "order",
        value_name = "ORDER",
        help = "The channel ordering, valid options 'unordered' (default) and 'ordered'",
        default_value_t
    )]
    order: Ordering,

    #[clap(
        long = "a-port",
        required = true,
        value_name = "A_PORT_ID",
        help_heading = "FLAGS",
        help = "Identifier of the side `a` port for the new channel"
    )]
    port_a: PortId,

    #[clap(
        long = "b-port",
        required = true,
        value_name = "B_PORT_ID",
        help_heading = "FLAGS",
        help = "Identifier of the side `b` port for the new channel"
    )]
    port_b: PortId,

    #[clap(
        long = "channel-version",
        visible_alias = "chan-version",
        value_name = "VERSION",
        help = "The version for the new channel"
    )]
    version: Option<Version>,
    //
    // #[clap(
    //     long = "root",
    //     required = true,
    //     value_name = "PATH",
    //     help_heading = "REQUIRED",
    //     help = "Path to the configuration file"
    //     )]
    // path: PathBuf,
}

#[derive(Clone, Command, Debug, Parser)]
pub struct ChannelsQueryCmd {
    #[clap(
        long = "source",
        required = true,
        value_name = "SOURCE_CHAIN_ID",
        help_heading = "REQUIRED",
        help = "Identifier of the source chain"
    )]
    pub(crate) source_chain_id: ChainId,
    #[clap(
        long = "dest",
        required = true,
        value_name = "DESTINATION_CHAIN_ID",
        help_heading = "REQUIRED",
        help = "Identifier of the destination chain"
    )]
    pub(crate) destination_chain_id: ChainId,
    #[clap(
        long = "ibc",
        help_heading = "FLAGS",
        help = "Get information using IBC protocol. If not used then REST API will be used"
    )]
    pub(crate) ibc: bool,
    #[clap(
        long = "status",
        help_heading = "FLAGS",
        help = "Give status: 0 if channels exists or 1 if not"
    )]
    pub(crate) status: bool,
    //
    // #[clap(
    //     long = "root",
    //     required = true,
    //     value_name = "PATH",
    //     help_heading = "REQUIRED",
    //     help = "Configuration file location"
    // )]
    // pub(crate) path: PathBuf,
}

impl Runnable for ChannelUpdateToConfigCmd {
    fn run(&self) {
        todo!()
    }
}

impl Runnable for ChannelsCreateCmd {
    fn run(&self) {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        // let home_dir = &self.path.parent().unwrap().to_path_buf();

        info!("Started creating channels:");
        info!("Source chain id: {}", self.source_chain_id);
        info!("Destination chain id: {}", self.destination_chain_id);
        info!("Home dir: {:?}", app_home());

        let res = rt.block_on(async {
            get_channels_all(
                &self.source_chain_id,
                &self.destination_chain_id,
                // home_dir,
            ).await
        });

        match res {
            Ok(x) => {
                if x.data.len() > 0 {
                    println!("Channels already exist:");
                    x.data.iter().for_each(|channel| {
                        println!(
                            "{} - {} ({})",
                            channel.source_channel_id,
                            channel.destination_channel_id,
                            channel.source_denom,
                        );
                    });
                } else {
                    println!("Channels do not exist. Creating channels...");
                    self.create_channels(
                        &self.source_chain_id,
                        &self.destination_chain_id,
                    );
                }
            }
            Err(err) => {
                println!("Error: {}", err);
            }
        }
    }
}

/// Create channels between the two chains using Hermes
impl ChannelsCreateCmd {
    pub fn create_channels(&self, chain_a: &ChainId, chain_b: &ChainId) {
        let chan_reg_full_path = get_channel_reg_file(chain_a.as_str());

        if !chan_reg_full_path.exists() {
            info!("Channel config file does not exist. Creating a new one: {:?}", &chan_reg_full_path);
            File::create(&chan_reg_full_path).expect("Unable to create file");
        } else {
            info!("Reading channel config from file: {:?}", &chan_reg_full_path);
        }

        let json_res = load_json::<ChannelPathInfo>(&chan_reg_full_path);
        let mut info = match json_res {
            Ok(x) => {
                if  x.chain.chain_id.as_str() != "" &&
                    x.chain.chain_id.as_str() != chain_a.as_str() {
                    info!("Chain A is different from the one in the config file. Updating config file");
                    Output::error("Chain A is different from the one in the config file. Updating config file")
                        .exit();
                }
                x
            },
            Err(err) => {
                error!("Error reading config file, returning default: {}", err);
                ChannelPathInfo::default()
            }
        };

        info.chain.chain_id = chain_a.to_string();

        info!("Reading configuration file");
        let config = app_config();

        let chains = ChainHandlePair::spawn(
            &config,
            chain_a,
            chain_b,
        ).unwrap_or_else(exit_with_unrecoverable_error);

        info!(
            "Creating new clients, new connection, and a new channel with order {}",
            self.order
        );

        // Create the clients.
        let client_a = ForeignClient::new(
            chains.src.clone(),
            chains.dst.clone(),
        ).unwrap_or_else(exit_with_unrecoverable_error);

        let client_b = ForeignClient::new(
            chains.dst.clone(),
            chains.src.clone(),
        ).unwrap_or_else(exit_with_unrecoverable_error);

        // Create the connection.
        let con = Connection::new(
            client_a.clone(),
            client_b.clone(),
            connection_delay(),
        ).unwrap_or_else(exit_with_unrecoverable_error);

        // Finally create the channel.
        let channel = Channel::new(
            con.clone(),
            self.order,
            self.port_a.clone(),
            self.port_b.clone(),
            self.version.clone(),
        ).unwrap_or_else(exit_with_unrecoverable_error);

        info!("Channel created successfully, saving to config file");
        info.channels.push(IBCPathInfo {
            chain: ChainShortInfo {
                chain_name: "".to_string(), // leaving is empty for now
                chain_id: chain_b.to_string(),
            },
            source: ChannelConfig {
                channel_id: channel.a_side.channel_id().unwrap().clone(),
                port_id: PortId::transfer(),
                client_id: client_a.id,
                connection_id: con.a_side.connection_id().unwrap().clone(),
            },
            target: ChannelConfig {
                channel_id: channel.b_side.channel_id().unwrap().clone(),
                port_id: PortId::transfer(),
                client_id: client_b.id,
                connection_id: con.b_side.connection_id().unwrap().clone(),
            },
        });

        match store_json(&info, &chan_reg_full_path) {
            Ok(_) => {
                info!("Channel config file updated successfully: {}", chan_reg_full_path.display());
            }
            Err(err) => {
                error!("Error updating channel config file: {}", err);
            }
        }

        Output::success(channel).exit();
    }
}

impl Runnable for ChannelsQueryCmd {
    fn run(&self) {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        let res = rt.block_on(async {
            self.get_channels_all().await
        });

        let data = res.unwrap();

        data.data.iter().for_each(|channel| {
            println!(
                "{} - {} ({})",
                channel.source_channel_id, channel.destination_channel_id, channel.base_denom.symbol
            );
        });

        if self.status && data.data.len() == 0 {
            Output::error("Channels do not exist").exit();
        } else {
            Output::success("Channels exist").exit();
        }
    }
}

impl ChannelsQueryCmd {
    async fn get_channels_from_tfm(&self) -> Result<Channels, RegistryError> {
        let url = Builder::new()
            .scheme(PROTOCOL)
            .authority(HOST)
            .path_and_query(
                format!(
                    "/channels/pairs?sourceChainId={}&destinationChainId={}&page=1&take=1000",
                    self.source_chain_id, self.destination_chain_id
                )
                .as_str(),
            )
            .build()
            .unwrap();

        info!("Querying TFM registry at {}", url);
        let response = reqwest::get(url.to_string())
            .await
            .map_err(|e| RegistryError::request_error(url.to_string(), e))?;

        if response.status().is_success() {
            match response.text().await {
                Ok(body) => match serde_json::from_str(&body) {
                    Ok(parsed) => Ok(parsed),
                    Err(e) => Err(RegistryError::json_parse_error(
                        self.source_chain_id.to_string(),
                        e,
                    )),
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

    async fn get_channels_from_file_registry(&self) -> Result<Channels, RegistryError> {
        let full_path = get_channel_reg_file(self.source_chain_id.as_str());
        let config_res = load_json::<ChannelPathInfo>(&full_path);
        info!("Query channels from file registry: {:?}", full_path);

        match config_res {
            Ok(info) => {
                let mut channels = Channels {
                    data: Vec::new()
                };

                info.channels.iter().for_each(|channel| {
                    if self.destination_chain_id.to_string() == channel.chain.chain_id {
                        let chi = ChannelInfo {
                            source_chain_id: Default::default(),
                            destination_chain_id: Default::default(),
                            source_channel_id: channel.source.channel_id.to_string(),
                            destination_channel_id: channel.target.channel_id.to_string(),
                            port_id: channel.source.port_id.to_string(),
                            destination_port_id: channel.target.port_id.to_string(),
                            source_denom: Default::default(),
                            destination_denom: Default::default(),
                            base_denom: Default::default(),
                        };

                        channels.data.push(chi);
                    }
                });

                Ok(channels)
            }
            Err(e) => {
                warn!("Error loading channel config file: {}", e);
                Err(RegistryError::registry_config_read_error(e.to_string()))
            }
        }
    }

    async fn get_channels_all(&self) -> Result<Channels, RegistryError> {
        let mut res: Vec<Result<Channels, RegistryError>> = Vec::new();

        res.push(self.get_channels_from_tfm().await);
        res.push(self.get_channels_from_file_registry().await);

        combine_results(res)
    }
}

fn get_channel_reg_file(chain: &str) -> PathBuf {
    let mut home = app_home();
    if !home.is_absolute() {
        home = home_dir().unwrap().join(home)
    }

    info!("Channel config file HOME dir: {:?}", home);
    let channels_file_name = format!("{chain}-channels.json").to_string();
    let full_path = home.join(&channels_file_name);
    info!("Channel config file: {:?}", full_path);
    full_path

    // if !full_path.exists() {
    //     info!("Channel config file does not exist. Creating a new one: {}", &full_path.to_str().unwrap());
    //     File::create(&full_path).expect("Unable to create file");
    // } else {
    //     info!("Reading channel config from file: {:?}", &full_path);
    // }
    //
    // full_path
}

// Get all channels from TFM and file registry and combine them
pub async fn get_channels_all(
    source: &ChainId,
    destination: &ChainId,
    // home_dir: &PathBuf,
) -> Result<Channels, RegistryError> {
    let cmd = ChannelsQueryCmd {
        source_chain_id: source.clone(),
        destination_chain_id: destination.clone(),
        ibc: false,
        status: false,
        // path: home_dir.clone(),
    };

    cmd.get_channels_all().await
}

fn combine_results(results: Vec<Result<Channels, RegistryError>>) -> Result<Channels, RegistryError> {
    let (ok_results, _): (Vec<_>, Vec<_>) =
        results.into_iter().partition(Result::is_ok);

    if ok_results.is_empty() {
        Err(RegistryError::no_channels_found())
    } else {
        let ok_values: Channels = ok_results
            .into_iter()
            .map(Result::unwrap)
            .fold(Channels::default(), |mut acc, mut x| {
                acc.data.append(&mut x.data);
                acc
            });

        Ok(ok_values)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_url() {
        let cmd = ChannelsQueryCmd {
            source_chain_id: ChainId::new("phoenix".to_string(), 1),
            destination_chain_id: ChainId::new("osmosis".to_string(), 1),
            ibc: false,
            status: false,
            // path: PathBuf::from(""),
        };

        let res = cmd.get_channels_from_tfm().await;

        assert_eq!(res.is_ok(), true);
        let channels = res.unwrap();
        assert!(channels.data.len() > 0);
    }
}
