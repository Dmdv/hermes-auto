//! Contains functions to generate a relayer config for a given chain

use crate::commands::channels::get_channels_all;
use futures::future::join_all;
use http::Uri;

use tracing::{
    error,
    info,
    warn
};

use std::{
    fmt::Display,
    collections::HashMap,
    marker::Send,
    thread,
    time,
    collections::HashSet
};

use core::str::FromStr;

use ibc_relayer::{
    config::{
        filter::{FilterPattern, PacketFilter},
        gas_multiplier::GasMultiplier,
        types::{MaxMsgNum, MaxTxSize, Memo},
        ChainTransport, {default, AddressType, ChainConfig, GasPrice, EventSourceMode},
        // load_json,
   },
    keyring::Store,
};

use itertools::Itertools;

use tendermint_light_client_verifier::types::TrustThreshold;
use tendermint_rpc::Url;

use ibc_chain_registry::{
    paths::ChainDesc,
    asset_list::AssetList,
    chain::ChainData,
    error::RegistryError,
    fetchable::Fetchable,
    formatter::{SimpleGrpcFormatter, UriFormatter},
    paths::{Channel, ChannelPort, IBCPath, ChannelPathInfo},
    querier::*,
};

use tokio::task::{JoinError, JoinHandle};

// use ibc_relayer_types::core::ics24_host::identifier::{ChannelId, ClientId, ConnectionId, PortId};
use ibc_relayer_types::core::ics24_host::identifier::{ChainId, ChannelId, PortId};

const MAX_HEALTHY_QUERY_RETRIES_EX: u8 = 5;

/// Generate packet filters from Vec<IBCPath> and load them in a Map(chain_name -> filter).
fn construct_packet_filters(ibc_paths: Vec<IBCPath>) -> HashMap<String, PacketFilter> {
    let mut packet_filters: HashMap<_, Vec<_>> = HashMap::new();

    for path in &ibc_paths {
        for channel in &path.channels {
            let chain_1 = path.chain_1.chain_name.to_owned();
            let chain_2 = path.chain_2.chain_name.to_owned();

            let filters_1 = packet_filters.entry(chain_1).or_default();

            filters_1.push((
                FilterPattern::Exact(channel.chain_1.port_id.clone()),
                FilterPattern::Exact(channel.chain_1.channel_id.clone()),
            ));

            let filters_2 = packet_filters.entry(chain_2).or_default();

            filters_2.push((
                FilterPattern::Exact(channel.chain_2.port_id.clone()),
                FilterPattern::Exact(channel.chain_2.channel_id.clone()),
            ));
        }
    }

    // remove duplicates from filters
    let packet_filters = packet_filters
        .into_iter()
        .map(|(k, v)| (k, v.into_iter().unique().collect::<Vec<_>>()))
        .collect::<HashMap<_, _>>();

    packet_filters
        .into_iter()
        .map(|(k, v)| (k, PacketFilter::allow(v)))
        .collect()
}

pub async fn chain_healthy_transport<GrpcQuerier, RpcQuerier, GrpcFormatter>(
    chain_data: ChainData,
) -> Result<TransportConfigData, RegistryError>
where
    GrpcQuerier:
        QueryContext<QueryInput = Uri, QueryOutput = Url, QueryError = RegistryError> + Send,
    RpcQuerier: QueryContext<
            QueryInput = String,
            QueryOutput = HermesConfigData,
            QueryError = RegistryError,
        > + Send,
    GrpcFormatter: UriFormatter<OutputFormat = Uri>,
{
    info!("Checking healthy endpoints for chain: {}", chain_data.chain_name);
    let chain_name = chain_data.chain_name;

    info!("grpc_endpoints checks");
    let grpc_endpoints = chain_data
        .apis
        .grpc
        .iter()
        .map(|grpc| GrpcFormatter::parse_or_build_address(grpc.address.as_str()))
        .collect::<Result<_, _>>()?;

    info!("rpc_endpoints checks");
    let rpc_endpoints: Vec<String> = chain_data
        .apis
        .rpc
        .iter()
        .map(|rpc| rpc.address.to_owned())
        .collect();

    info!("Starting rpc queries");
    let rpc_data_res =
        RpcQuerier::query_healthy_all(chain_name.to_string(), rpc_endpoints).await;
    info!("Starting grpc queries");
    let grpc_address_res =
        GrpcQuerier::query_healthy_all(chain_name.to_string(), grpc_endpoints).await;

    let rpc_data = rpc_data_res.unwrap_or_else(|e| {
        warn!("Not found healthy RPC endpoints: {}", e);
        Vec::default()
    });

    let grpc_data = grpc_address_res.unwrap_or_else(|e| {
        warn!("Not found healthy GRPC endpoints: {}", e);
        Vec::default()
    });

    Ok(TransportConfigData {
        rpc: rpc_data,
        grpc: grpc_data,
    })
}

pub async fn chain_transport<GrpcQuerier, RpcQuerier, GrpcFormatter>(
    chain_data: ChainData,
) -> Result<ChainTransport, RegistryError>
where
    GrpcQuerier:
        QueryContext<QueryInput = Uri, QueryOutput = Url, QueryError = RegistryError> + Send,
    RpcQuerier: QueryContext<
            QueryInput = String,
            QueryOutput = HermesConfigData,
            QueryError = RegistryError,
        > + Send,
    GrpcFormatter: UriFormatter<OutputFormat = Uri>,
{
    let chain_name = chain_data.chain_name;

    let grpc_endpoints = chain_data
        .apis
        .grpc
        .iter()
        .map(|grpc| GrpcFormatter::parse_or_build_address(grpc.address.as_str()))
        .collect::<Result<_, _>>()?;

    let rpc_endpoints: Vec<String> = chain_data
        .apis
        .rpc
        .iter()
        .map(|rpc| rpc.address.to_owned())
        .collect();

    let rpc_data: HermesConfigData = query_healthy_retry::<RpcQuerier>(
        chain_name.to_string(),
        rpc_endpoints,
        MAX_HEALTHY_QUERY_RETRIES_EX,
    ).await?;

    let grpc_address : Url = query_healthy_retry::<GrpcQuerier>(
        chain_name.to_string(),
        grpc_endpoints,
        MAX_HEALTHY_QUERY_RETRIES_EX,
    ).await?;

    let websocket_address =
        rpc_data.websocket.clone().try_into().map_err(|e| {
            RegistryError::websocket_url_parse_error(rpc_data.websocket.to_string(), e)
        })?;

    Ok(ChainTransport {
        rpc_addr: rpc_data.rpc_address,
        websocket_addr: websocket_address,
        grpc_addr: grpc_address,
    })
}

/// Generates a ChainConfig for a given chain from ChainData, AssetList, and an optional PacketFilter.
async fn hermes_config<GrpcQuerier, RpcQuerier, GrpcFormatter>(
    chain_data: ChainData,
    assets: AssetList,
    packet_filter: Option<PacketFilter>,
) -> Result<ChainConfig, RegistryError>
where
    GrpcQuerier:
        QueryContext<QueryInput = Uri, QueryOutput = Url, QueryError = RegistryError> + Send,
    RpcQuerier: QueryContext<
            QueryInput = String,
            QueryOutput = HermesConfigData,
            QueryError = RegistryError,
        > + Send,
    GrpcFormatter: UriFormatter<OutputFormat = Uri>,
{
    let chain_name = chain_data.chain_name;

    let asset = assets
        .assets
        .first()
        .ok_or_else(|| RegistryError::no_asset_found(chain_name.to_string()))?;

    let grpc_endpoints: Vec<Uri> = chain_data
        .apis
        .grpc
        .iter()
        .map(|grpc| GrpcFormatter::parse_or_build_address(grpc.address.as_str()))
        .collect::<Result<_, _>>()?;

    let rpc_endpoints: Vec<String> = chain_data
        .apis
        .rpc
        .iter()
        .map(|rpc| rpc.address.to_owned())
        .collect();

    info!("Starting RPC queries for chain: {}", chain_name);
    let rpc_data: HermesConfigData = query_healthy_retry::<RpcQuerier>(
        chain_name.to_string(),
        rpc_endpoints.clone(),
        MAX_HEALTHY_QUERY_RETRIES_EX,
    ).await?;

    info!("Starting gRPC queries for chain: {}", chain_name);
    let grpc_address : Result<Url, RegistryError> = query_healthy_retry::<GrpcQuerier>(
        chain_name.to_string(),
        grpc_endpoints.clone(),
        MAX_HEALTHY_QUERY_RETRIES_EX,
    ).await;

    let grpc_any = match grpc_address {
        Ok(grpc_address) => {
            info!("Chain: {}, found first healthy gRPC: {}", chain_name, grpc_address);
            grpc_address
        }
        Err(e) => {
            if grpc_endpoints.is_empty() {
                let default_grpc = rpc_data.rpc_address.clone();
                info!(
                    "Chain: {}, doesn't have gRPC endpoints: {}. If will be defaulted to the rpc: {}",
                    chain_name, e, default_grpc
                );
                default_grpc
            } else {
                let default_grpc = grpc_endpoints.first().unwrap().to_string().parse().unwrap();
                info!(
                    "Chain: {}, doesn't have healthy grpc endpoint: {}. If will be defaulted to: {}",
                    chain_name, e, default_grpc
                );
                default_grpc
            }
        }
    };

    let websocket_address =
        rpc_data.websocket.clone().try_into().map_err(|e| {
            RegistryError::websocket_url_parse_error(rpc_data.websocket.to_string(), e)
        })?;

    Ok(ChainConfig {
        id: chain_data.chain_id,
        r#type: default::chain_type(),
        rpc_addr: rpc_data.rpc_address,
        event_source: EventSourceMode::Push {
            url: websocket_address,
            batch_delay: default::batch_delay(),
        },
        grpc_addr: grpc_any,
        rpc_timeout: default::rpc_timeout(),
        trusted_node: true,
        genesis_restart: None,
        account_prefix: chain_data.bech32_prefix,
        key_name: String::new(),
        key_store_type: Store::default(),
        key_store_folder: None,
        store_prefix: "ibc".to_string(),
        default_gas: Some(100000),
        max_gas: Some(400000),
        gas_adjustment: None,
        gas_multiplier: Some(GasMultiplier::new(1.1).unwrap()),
        fee_granter: None,
        max_msg_num: MaxMsgNum::default(),
        max_tx_size: MaxTxSize::default(),
        max_grpc_decoding_size: default::max_grpc_decoding_size(),
        clock_drift: default::clock_drift(),
        max_block_time: default::max_block_time(),
        trusting_period: None,
        ccv_consumer_chain: false,
        memo_prefix: Memo::default(),
        proof_specs: Default::default(),
        trust_threshold: TrustThreshold::default(),
        gas_price: GasPrice {
            price: 0.1,
            denom: asset.base.to_owned(),
        },
        packet_filter: packet_filter.unwrap_or_default(),
        address_type: AddressType::default(),
        sequential_batch_tx: false,
        extension_options: Vec::new(),
    })
}

/// Concurrent `query_healthy` might fail, this is a helper function which will retry a failed query a fixed
/// amount of times in order to avoid failure with healthy endpoints.
async fn query_healthy_retry<QuerierType>(
    chain_name: String,
    endpoints: Vec<QuerierType::QueryInput>,
    retries: u8,
) -> Result<QuerierType::QueryOutput, RegistryError>
where
    QuerierType: QueryContext + Send,
    QuerierType::QueryInput: Clone + Display,
    QuerierType: QueryContext<QueryError = RegistryError>,
{
    info!(
        "Querying chain `{}` with {} retries",
        chain_name, retries
    );

    for i in 0..retries {
        let query_response =
            QuerierType::query_healthy(chain_name.to_string(), endpoints.clone()).await;
        match query_response {
            Ok(r) => {
                info!("Query of {} succeeded with response {:?}", &chain_name, r);
                return Ok(r);
            }
            Err(_) => {
                info!("Retry {i} failed to query {} endpoints", &chain_name);
            }
        }
    }

    Err(RegistryError::unhealthy_endpoints(
        endpoints
            .iter()
            .map(|endpoint| endpoint.to_string())
            .collect(),
        retries,
        chain_name,
    ))
}

async fn get_handles<T: Fetchable + Send + 'static>(
    resources: &[String],
    commit: &Option<String>,
) -> Vec<JoinHandle<Result<T, RegistryError>>> {
    let handles = resources
        .iter()
        .map(|resource| {
            let resource = resource.to_string();
            let commit = commit.clone();
            tokio::spawn(async move { T::fetch(resource, commit).await })
        })
        .collect();
    handles
}

async fn get_data_from_handles<T>(
    handles: Vec<JoinHandle<Result<T, RegistryError>>>,
    error_task: &str,
) -> Result<Vec<T>, RegistryError> {
    let data_array: Result<Vec<_>, JoinError> = join_all(handles).await.into_iter().collect();
    let data_array: Result<Vec<T>, RegistryError> = data_array
        .map_err(|e| RegistryError::join_error(error_task.to_string(), e))?
        .into_iter()
        .collect();
    data_array
}

// /// Attempt to load and parse the Json config file as a `Config`.
// pub fn get_channels_config(full_path: impl AsRef<Path>) -> Result<ChannelPathInfo, RegistryError> {
//     let config_json = fs::read_to_string(&path).map_err(RegistryError::io)?;
//     match serde_json::from_str(&config_json) {
//         Ok(config) => Ok(config),
//         Err(e) => {
//             error!("Error parsing channel config file: {}", e);
//             Err(RegistryError::json_load_error( e))
//         }
//     }
// }

/// Generates a `Vec<ChainConfig>` for a slice of chains names by fetching data from
/// <https://github.com/cosmos/chain-registry>. Gas settings are set to default values.
///
/// # Arguments
///
/// * `chains` - A slice of strings that holds the name of the chains for which a `ChainConfig` will be generated. It must be sorted.
/// * `commit` - An optional String representing the commit hash from which the chain configs will be generated. If it's None, the latest commit will be used.
///
/// # Example
///
/// ```
/// use std::path::PathBuf;
/// use ibc_relayer_cli::chain_registry_ex::get_configs;
/// let chains = &vec!["cosmoshub".to_string(), "osmosis".to_string()];
/// let configs = get_configs(chains, None, None);
/// ```
pub async fn get_configs(
    chains: &[String],
    source_chain: Option<String>, // source chain from which to fetch the configs
    commit: Option<String>,
) -> Result<Vec<ChainConfig>, RegistryError> {
    let n = chains.len();
    if n == 0 {
        return Ok(Vec::new());
    }

    let from_chain = source_chain.map(|s| s.to_lowercase());
    let chains : Vec<String> = chains.iter().map(|s| s.to_lowercase()).collect();

    info!("Getting configuration...");
    info!("Chains: '{:?}'", chains);
    info!("Source chain: '{}'", from_chain.clone().unwrap_or_default());

    let threads: Vec<Vec<String>> = chains.chunks(5).map(|chunk| chunk.into()).collect();
    let mut chain_data: Vec<ChainData> = Vec::new();
    let mut asset_data: Vec<AssetList> = Vec::new();

    for v in threads.iter() {
        // Spawn tasks to fetch data from the chain-registry
        let chain_data_handle = get_handles::<ChainData>(v, &commit).await;
        let asset_lists_handle = get_handles::<AssetList>(v, &commit).await;

        thread::sleep(time::Duration::from_millis(100));

        // Collect data from the spawned tasks
        let chain_data_array =
            get_data_from_handles::<ChainData>(chain_data_handle, "chain_data_join").await?;
        let assets_array =
            get_data_from_handles::<AssetList>(asset_lists_handle, "asset_handle_join").await?;

        chain_data.extend(chain_data_array);
        asset_data.extend(assets_array);
    }

    // Creating dictionary to find chain_id by chain name

    let dic = chain_data
        .clone()
        .into_iter()
        .fold(
            HashMap::new(),
            |mut acc, chain_data| {
                acc.insert(
                    chain_data.chain_name.to_owned(),
                    chain_data.chain_id.to_owned(),
                );
                acc
            }
        );

    // Hashset of chains to lowercase

    let chain_set = chains.as_slice().into_iter()
        .fold(HashSet::new(), |mut acc, chain_name| {
            acc.insert(chain_name.to_lowercase());
            acc
        });

    // Fetching paths from chain-registry
    let mut path_handles = Vec::with_capacity(n * (n - 1) / 2);

    if let Some(source) = &from_chain {
        let source = source.to_lowercase();
        if !chain_set.contains(source.as_str()) {
            return Err(RegistryError::no_chain_found(source.clone()));
        }

        for chain in chains.as_slice() {
            if chain == &source {
                continue;
            }
            let resource = format!("{source}-{chain}.json").to_string();
            let commit_clone = commit.clone();
            path_handles.push(tokio::spawn(async move {
                IBCPath::fetch(resource, commit_clone).await
            }));
        }
    } else {
        for i in 0..n {
            for chain_j in &chains[i + 1..] {
                let chain_i = &chains[i];
                let resource = format!("{chain_i}-{chain_j}.json").to_string();
                let commit_clone = commit.clone();
                path_handles.push(tokio::spawn(async move {
                    IBCPath::fetch(resource, commit_clone).await
                }));
            }
        }
    }

    // Collect path data from default github registry
    let path_data: Result<Vec<_>, JoinError> = join_all(path_handles).await.into_iter().collect();
    let mut path_data: Vec<IBCPath> = path_data
        .map_err(|e| RegistryError::join_error("path_handle_join".to_string(), e))?
        .into_iter()
        .filter_map(|path| path.ok())
        .collect();

    // if source_chain is provided, we need to fetch the channel config file and from tfm data
    if let Some(source) = &from_chain {
        // let source = source.to_lowercase();
        // let chain_id = dic.get(&source).ok_or_else(ChainId::default).unwrap();
        // let channels_file_name = format!("{chain_id}-channels.json").to_string();
        // let full_path = home_dir.join(channels_file_name);
        // info!("Source chain is provided, reading data from {:?}", full_path);

        // load_json::<ChannelPathInfo>(&full_path);
        // From channel config file
        // match get_channels_config(&full_path) {
        //     Ok(cfg) => {
        //         info!("Channel config file found at {:?}, read data from channel registry", full_path);
        //         construct_paths_from_channel_config(
        //             &mut path_data,
        //             &chain_set,
        //             &source,
        //             &cfg
        //         ).await;
        //     },
        //     Err(_e) => {
        //         warn!("No channel config file found at {:?}, reading data from channel registry will be skipped", full_path);
        //     }
        // };

        construct_from_tfm_data_to_source(chains.as_slice(), &mut path_data, &dic, &source).await;
    } else {
        construct_from_tfm_data(chains.as_slice(), &mut path_data, &dic).await;
    }

    let mut packet_filters = construct_packet_filters(path_data);

    // DD: this is a workaround for the above code that does not work with large amount of chains (>5)
    let mut configs: Vec<Result<ChainConfig, RegistryError>> = Vec::new();

    let config_handles = chain_data
        .into_iter()
        .zip(asset_data.into_iter())
        .zip(chains.iter());

    for ((cd, assets), chain_name) in config_handles {
        let packet_filter = packet_filters.remove(chain_name);
        info!("Creating config for chain {}", chain_name);
        let res = hermes_config::<
            GrpcHealthCheckQuerier,
            SimpleHermesRpcQuerier,
            SimpleGrpcFormatter,
        >(cd, assets, packet_filter)
        .await;

        configs.push(res);
    }

    let data_array: Result<Vec<ChainConfig>, RegistryError> = configs.into_iter().collect();

    data_array
}

// get paths only from a specific chain as source, for example, from terra2 to all other chains
async fn construct_from_tfm_data_to_source(
    chains: &[String],
    path_data: &mut Vec<IBCPath>,
    dic: &HashMap<String, ChainId>,
    from: &str
) {
    info!("Adding packet filters from TFM endpoint with source chain: {}", from);

    for chain_name in chains {
        if chain_name == from {
            continue;
        }
        build_paths(path_data, dic, from, &chain_name).await;
    }
}

async fn construct_from_tfm_data(
    chains: &[String],
    path_data: &mut Vec<IBCPath>,
    dic: &HashMap<String, ChainId>
) {
    let n = chains.len();
    info!("Adding packet filters from TFM endpoint without source chain");
    for i in 0..n {
        let chain_a = &chains[i];
        for chain_b in &chains[i + 1..] {
            build_paths(path_data, dic, chain_a, chain_b).await;
        }
    }
}

async fn _construct_paths_from_channel_config(
    path_data: &mut Vec<IBCPath>,
    chains: &HashSet<String>,
    source_chain: &str,
    cfg: &ChannelPathInfo) {

    info!("Adding packet filters from channels config file");

    if source_chain != &cfg.chain.chain_name {
        warn!("Source chain {} does not match channel config source chain {}",
              source_chain,
            cfg.chain.chain_name);
        return;
    }

    info!("Adding packet filters from channel config file {} for {}",
        format!("{source_chain}-channels.json"),
        &source_chain);

    let chain_a = &cfg.chain.chain_name;
    let mut cd1 = ChainDesc::default();
    cd1.chain_name = chain_a.clone();

    for channel in &cfg.channels {
        if !chains.contains(&channel.chain.chain_name.to_lowercase()) {
            continue;
        }

        let mut path = IBCPath::default();

        let chain_b = &channel.chain.chain_name;
        let mut cd2 = ChainDesc::default();
        cd2.chain_name = chain_b.clone();

        info!("Source chain: {}, destination chain: {}", chain_a, chain_b);

        path.chain_1 = cd1.clone();
        path.chain_2 = cd2.clone();

        let mut channel_desc = Channel::default();
        channel_desc.chain_1 = ChannelPort {
            port_id: PortId::transfer(),
            channel_id: ChannelId::from_str(channel.source.channel_id.as_str()).unwrap(),
        };
        channel_desc.chain_2 = ChannelPort {
            port_id: PortId::transfer(),
            channel_id: ChannelId::from_str(channel.target.channel_id.as_str()).unwrap(),
        };

        info!(
            "Found channels for {} -> {} = [ {} -> {} ]",
            chain_a,
            chain_b,
            channel.source.channel_id,
            channel.target.channel_id,
        );

        path.channels.push(channel_desc);

        path_data.push(path.clone());
    }
}

async fn build_paths(
    path_data: &mut Vec<IBCPath>,
    dic: &HashMap<String, ChainId>,
    chain_a: &str,
    chain_b: &str
) {
    info!("Building paths for {} -> {}", chain_a, chain_b);

    let chain_a_id = dic
        .get(chain_a)
        .expect(format!("chain a {} not found. Add this to the chain list", chain_a).as_str());
    let chain_b_id = dic
        .get(chain_b)
        .expect(format!("chain b {} not found. Add this to the chain list", chain_b).as_str());

    info!(
        "Looking for {} -> {} ({} -> {})",
        chain_a, chain_b, chain_a_id, chain_b_id
    );

    let channels = get_channels_all(
        &ChainId::from_str(chain_a_id.as_str()).unwrap(),
        &ChainId::from_str(chain_b_id.as_str()).unwrap()
    )
    .await;

    match channels {
        Ok(ch) => {
            let mut path = IBCPath::default();

            let mut cd1 = ChainDesc::default();
            cd1.chain_name = chain_a.to_string();
            let mut cd2 = ChainDesc::default();
            cd2.chain_name = chain_b.to_string();

            path.chain_1 = cd1;
            path.chain_2 = cd2;

            // Endpoints returns duplicate channels for different tokens
            // Remove duplicates from channels
            let unique_channels = ch
                .data
                .into_iter()
                .unique_by(|channel| {
                    (
                        channel.source_channel_id.clone(),
                        channel.destination_channel_id.clone(),
                    )
                })
                .collect::<Vec<_>>();

            for channel in unique_channels.iter() {
                info!(
                    "Found channels for {} -> {} = [ {} -> {} ]",
                    chain_a_id,
                    chain_b_id,
                    channel.source_channel_id,
                    channel.destination_channel_id,
                );

                let mut channel_desc = Channel::default();
                channel_desc.chain_1 = ChannelPort {
                    port_id: PortId::transfer(),
                    channel_id: ChannelId::from_str(channel.source_channel_id.as_str()).unwrap(),
                };
                channel_desc.chain_2 = ChannelPort {
                    port_id: PortId::transfer(),
                    channel_id: ChannelId::from_str(channel.destination_channel_id.as_str())
                        .unwrap(),
                };

                path.channels.push(channel_desc);
            }

            path_data.push(path);
        }
        Err(e) => {
            error!(
                "Error getting channels for chain {} and chain {}: {}",
                chain_a, chain_b, e
            );
        }
    }
}

/// Concurrent RPC and GRPC queries are likely to fail.
/// Since the RPC and GRPC endpoints are queried to confirm they are healthy,
/// before generating the ChainConfig, the tests must not all run concurrently or
/// else they will fail due to the amount of concurrent queries.
#[cfg(test)]
mod tests {
    use super::*;
    use ibc_relayer::config::filter::ChannelPolicy;
    use ibc_relayer_types::core::ics24_host::identifier::{ChannelId, PortId};
    use serial_test::serial;
    use std::str::FromStr;

    // Use commit from 28.04.23 for tests
    const TEST_COMMIT: &str = "95b99457e828402bde994816ce57e548d7e1a76d";

    // Helper function for configs without filter. The configuration doesn't have a packet filter
    // if there is no `{chain-a}-{chain-b}.json` file in the `_IBC/` directory of the
    // chain-registry repository: https://github.com/cosmos/chain-registry/tree/master/_IBC
    async fn should_have_no_filter(test_chains: &[String]) -> Result<(), RegistryError> {
        let configs = get_configs(
            test_chains,
            None,
            Some(TEST_COMMIT.to_owned())
        ).await?;

        for config in configs {
            match config.packet_filter.channel_policy {
                ChannelPolicy::AllowAll => {}
                _ => panic!("PacketFilter not allowed"),
            }
        }

        Ok(())
    }

    #[tokio::test]
    #[serial]
    async fn fetch_chain_config_with_packet_filters() -> Result<(), RegistryError> {
        let test_chains: &[String] = &[
            "cosmoshub".to_string(),
            "juno".to_string(),
            "osmosis".to_string(),
        ]; // Must be sorted

        let configs = get_configs(
            test_chains,
            None,
            Some(TEST_COMMIT.to_owned())
        ).await?;

        for config in configs {
            match config.packet_filter.channel_policy {
                ChannelPolicy::Allow(channel_filter) => {
                    if config.id.as_str().contains("cosmoshub") {
                        assert!(channel_filter.is_exact());

                        let cosmoshub_juno = (
                            &PortId::from_str("transfer").unwrap(),
                            &ChannelId::from_str("channel-207").unwrap(),
                        );

                        let cosmoshub_osmosis = (
                            &PortId::from_str("transfer").unwrap(),
                            &ChannelId::from_str("channel-141").unwrap(),
                        );

                        assert!(channel_filter.matches(cosmoshub_juno));
                        assert!(channel_filter.matches(cosmoshub_osmosis));
                        assert_eq!(channel_filter.len(), 2);
                    } else if config.id.as_str().contains("juno") {
                        assert!(channel_filter.is_exact());

                        let juno_cosmoshub = (
                            &PortId::from_str("transfer").unwrap(),
                            &ChannelId::from_str("channel-1").unwrap(),
                        );

                        let juno_osmosis_1 = (
                            &PortId::from_str("transfer").unwrap(),
                            &ChannelId::from_str("channel-0").unwrap(),
                        );

                        let juno_osmosis_2 = (
                            &PortId::from_str("wasm.juno1v4887y83d6g28puzvt8cl0f3cdhd3y6y9mpysnsp3k8krdm7l6jqgm0rkn").unwrap(),
                            &ChannelId::from_str("channel-47").unwrap()
                        );

                        assert!(channel_filter.matches(juno_cosmoshub));
                        assert!(channel_filter.matches(juno_osmosis_1));
                        assert!(channel_filter.matches(juno_osmosis_2));
                        assert_eq!(channel_filter.len(), 3);
                    } else if config.id.as_str().contains("osmosis") {
                        assert!(channel_filter.is_exact());

                        let osmosis_cosmoshub = (
                            &PortId::from_str("transfer").unwrap(),
                            &ChannelId::from_str("channel-0").unwrap(),
                        );

                        let osmosis_juno_1 = (
                            &PortId::from_str("transfer").unwrap(),
                            &ChannelId::from_str("channel-42").unwrap(),
                        );

                        let osmosis_juno_2 = (
                            &PortId::from_str("transfer").unwrap(),
                            &ChannelId::from_str("channel-169").unwrap(),
                        );

                        assert!(channel_filter.matches(osmosis_cosmoshub));
                        assert!(channel_filter.matches(osmosis_juno_1));
                        assert!(channel_filter.matches(osmosis_juno_2));
                        assert_eq!(channel_filter.len(), 3);
                    } else {
                        panic!("Unknown chain");
                    }
                }
                _ => panic!("PacketFilter not allowed"),
            }
        }

        Ok(())
    }

    #[tokio::test]
    #[serial]
    async fn fetch_chain_config_without_packet_filters() -> Result<(), RegistryError> {
        // The commit from 28.04.23 does not have `evmos-juno.json` nor `juno-evmos.json` file:
        // https://github.com/cosmos/chain-registry/tree/master/_IBC
        let test_chains: &[String] = &["evmos".to_string(), "juno".to_string()]; // Must be sorted
        should_have_no_filter(test_chains).await
    }

    #[tokio::test]
    #[serial]
    async fn fetch_one_chain() -> Result<(), RegistryError> {
        let test_chains: &[String] = &["cosmoshub".to_string()]; // Must be sorted
        should_have_no_filter(test_chains).await
    }

    #[tokio::test]
    #[serial]
    async fn fetch_no_chain() -> Result<(), RegistryError> {
        let test_chains: &[String] = &[];

        let configs = get_configs(
            test_chains,
            None,
            Some(TEST_COMMIT.to_owned())
        ).await?;

        assert_eq!(configs.len(), 0);

        Ok(())
    }
}
