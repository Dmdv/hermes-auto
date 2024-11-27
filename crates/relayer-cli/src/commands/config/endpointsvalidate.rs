use core::str::FromStr;
use crate::prelude::*;
use abscissa_core::clap::Parser;
use abscissa_core::{Command, Runnable};
use std::{
    fs,
    sync::Arc,
    thread,
    time,
};
use dirs;
use serde::{Deserialize, Serialize};
use tendermint_rpc::{WebSocketClientUrl, Url};

use crate::cli_utils::spawn_runtime;
use tokio::runtime::Runtime as TokioRuntime;

use crate::{
    chain_registry_ex::{
        chain_healthy_transport,
    },
    conclude::Output,
    config,
    config::config_path,
};

use ibc_relayer::{
    chain::endpoint::HealthCheck::*,
    chain::handle::ChainHandle,
    config::{default,
             store, store_json, load_json,
             types::Memo, ChainConfig, EventSourceMode, GasPrice},
};

use ibc_chain_registry::{
    chain::ChainData,
    error::RegistryError,
    fetchable::Fetchable,
    formatter::SimpleGrpcFormatter,
    paths::ChainIdMap,
    querier::*,
};

/// In order to validate the configuration file the command will check that the file exists,
/// that it is readable and not empty. It will then check the validity of the fields inside
/// the file.
#[derive(Command, Debug, Parser)]
pub struct HealthyEndpointsCmd {
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct RpcConfig {
    rpc: Url,
    websocket: Url,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct ChainLastConfig {
    grpc: Url,
    rpc: RpcConfig
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct ChainHealthyEndpoints {
    name: String,
    endpoints: TransportConfigData,
    last_config: ChainLastConfig,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct HealthyEndpoints {
    chains: Vec<ChainHealthyEndpoints>,
}

impl Default for HealthyEndpoints {
    fn default() -> Self {
        HealthyEndpoints {
            chains: vec![]
        }
    }
}

impl Runnable for HealthyEndpointsCmd {
    fn run(&self) {
        let config = app_config();
        trace!("loaded configuration: {:#?}", *config);

        // Verify that the configuration file has been found.
        match config_path() {
            Some(p) => {
                // If there is a configuration file, verify that it is readable and not empty.
                match fs::read_to_string(p.clone()) {
                    Ok(content) => {
                        if content.is_empty() {
                            Output::error("the configuration file is empty").exit();
                        }
                    }
                    Err(e) => {
                        Output::error(format!("error reading the configuration file {p:?}: {e}"))
                            .exit()
                    }
                }
            }
            None => Output::error("no configuration file found").exit(),
        }

        let path = config_path().unwrap();
        let home = path.parent().unwrap();
        info!("HOME folder {home:?}");

        let chain_name_map = {
            let home_dir = dirs::home_dir().unwrap();
            let chain_map_path = home_dir.join(".hermes/chain_chainid.json");
            if !chain_map_path.exists() {
                let msg = format!("chain map file {:?} does not exist, please update it", chain_map_path);
                info!(msg);
                Output::error(&msg).exit()
            } else {
                load_json::<ChainIdMap>(&chain_map_path)
            }
        }.unwrap();

        let chain_id_map = chain_name_map.inverse();

        // No need to output the underlying error, this is done already when the application boots.
        // See `application::CliApp::after_config`.
        match config::validate_config(&config) {
            Ok(_) => info!("Completed validating configuration"),
            Err(_) => Output::error("configuration is invalid").exit(),
        }

        info!("Starting endpoints validation");

        let mut new_config = config.clone();
        let cfg = Arc::make_mut(&mut new_config);
        let mut endpoints_config = HealthyEndpoints::default();

        for chain in cfg.chains.iter_mut() {
            chain.ccv_consumer_chain = default::ccv_consumer_chain();
            chain.rpc_timeout = default::rpc_timeout();
            chain.memo_prefix = Memo::default();

            let name: String;
            if let Some(chain_name) = chain_id_map.get(chain.id.as_str()) {
                name = chain_name.into();
            } else {
                warn!("Chain {:?} was not found in the chain id map", chain.id.as_str());
                warn!("This chain endpoints will not be updated");
                continue;
            }

            info!("*****************************************************************");
            info!("           Validating chain: {:?}", name);
            info!("*****************************************************************");
            info!("Fetching available chain data from registry for: {:?}",name);

            let rt = Arc::new(TokioRuntime::new().unwrap());

            let chain_data = rt.block_on(async {
                match ChainData::fetch(name.to_string(),None).await {
                    Ok(chain_data) => chain_data,
                    Err(e) => Output::error(format!(
                        "Error fetching chain data for chain {}: {}",
                        name, e
                    ))
                    .exit(),
                }
            });

            // Fixing the gas price which is missing in Hermes by default
            if &chain_data.fees.fee_tokens.len() > &0 {
                let token = chain_data.fees.fee_tokens.first().unwrap();
                chain.gas_price = GasPrice::new(token.high_gas_price, token.denom.clone());
            }

            info!("Fetching reachable endpoints for {:?}...", name);

            let rt = TokioRuntime::new().unwrap();

            let transport = rt.block_on(async {
                chain_healthy_transport::<
                    GrpcHealthCheckQuerier,
                    SimpleHermesRpcQuerier,
                    SimpleGrpcFormatter,
                >(chain_data)
                .await
            });

            let tr = transport.as_ref().unwrap();

            info!("Listing endpoints for {:?}", name);
            info!("RPC: -----------------");
            for rpc_data in &tr.rpc {
                info!("   {}", rpc_data.rpc_address);
            }

            info!("WS: -----------------");
            for rpc_data in &tr.rpc {
                info!("   {}", rpc_data.websocket);
            }

            info!("GRPC: -----------------");
            for grpc in &tr.grpc {
                info!("   {}", grpc);
            }

            if &tr.grpc.len() == &0 {
                info!("   No GRPC endpoints found");
            }

            // Defining endpoints config for this chain

            let rpc_default = &tr.rpc.first().unwrap();
            let last_config = ChainLastConfig {
                rpc: RpcConfig {
                    rpc: rpc_default.rpc_address.clone(),
                    websocket: rpc_default.websocket.clone(),
                },
                grpc: tr.grpc.first().unwrap().clone()
            };

            let mut endpoints = ChainHealthyEndpoints {
                name: name.clone(),
                endpoints: tr.clone(),
                last_config
            };

            info!("---------------------------------------");

            info!("Performing health check for {:?}...", name);

            let healthy_config = match &transport {
                Ok(transport_data) => {
                    info!("Selecting best configuration for {:?}", name);

                    let mut chain_config: Option<ChainConfig> = None;

                    // iterate rpc endpoints
                    'outer: for rpc_data in &transport_data.rpc {
                        let mut chain_ut = chain.clone();

                        let websocket_address: WebSocketClientUrl =
                            match rpc_data.websocket.clone().try_into() {
                                Ok(ws) => ws,
                                Err(e) => {
                                    error!("failed to parse websocket address, reason: {}", e);
                                    continue;
                                }
                            };

                        // iterate grpc endpoints
                        'inner: for grpc in &transport_data.grpc {
                            chain_ut.grpc_addr = grpc.clone();
                            chain_ut.rpc_addr = rpc_data.rpc_address.clone();
                            chain_ut.event_source = EventSourceMode::Push {
                                url: websocket_address.clone(),
                                batch_delay: default::batch_delay(),
                            };

                            info!("Performing health check for the following configuration...");
                            info!("*  Gas price: {}", chain_ut.gas_price);
                            info!("****************************************************");
                            info!("*  RPC: {}", chain_ut.rpc_addr);
                            info!("*  gRPC: {}", chain_ut.grpc_addr);
                            info!("*  Websocket: {}", websocket_address.clone());
                            info!("****************************************************");

                            thread::sleep(time::Duration::from_millis(1000));

                            let rt = match spawn_runtime(&chain_ut, None) {
                                Ok(rt) => rt,
                                Err(e) => {
                                    error!("failed to spawn runtime, reason: {}", e);
                                    continue 'inner;
                                }
                            };

                            match rt.health_check() {
                                Ok(Healthy) => {
                                    info!("Chain is healthy, using current configuration");
                                    chain_config = Some(chain_ut);
                                    break 'outer;
                                }
                                Ok(Unhealthy(_)) => {
                                    // No need to print the error here as it's already printed in `Chain::health_check`
                                    // TODO(romac): Move the printing code here and in the supervisor/registry
                                    // warn!("chain is not healthy")
                                    continue 'inner;
                                }
                                Err(e) => {
                                    error!(
                                        "failed to perform health check, reason: {}",
                                        e.detail()
                                    );
                                    continue 'inner;
                                }
                            }
                        }
                    }

                    if let Some(cg_cfg) = chain_config {
                        Ok(cg_cfg)
                    } else {
                        warn!("For {} was not found healthy endpoints", name);
                        Err(RegistryError::no_healthy_grpc)
                    }
                }
                Err(e) => {
                    warn!(
                        "For {} was not found healthy endpoints, reason: {}",
                        name,
                        e.detail()
                    );
                    Err(RegistryError::no_healthy_grpc)
                }
            };

            match healthy_config {
                Ok(c) => {
                    chain.rpc_addr = c.rpc_addr;
                    chain.grpc_addr = c.grpc_addr;
                    chain.event_source = c.event_source;

                    let websocket_url = match &chain.event_source {
                        EventSourceMode::Push { url, .. } => url.to_string(),
                        _ => panic!("No websocket url found")
                    };

                    endpoints.last_config = ChainLastConfig {
                        rpc: RpcConfig {
                            rpc: chain.rpc_addr.clone(),
                            websocket: Url::from_str(websocket_url.as_str()).unwrap(),
                        },
                        grpc: chain.grpc_addr.clone(),
                    };

                    info!("Healthy endpoints found for {}:", name);
                    info!("=========================================");
                    info!("=  RPC: {}", &chain.rpc_addr);
                    info!("=  gRPC: {}", &chain.grpc_addr);
                    info!("=  Websocket: {}", websocket_url);
                    info!("=========================================");
                }
                Err(_) => {
                    warn!(
                        "For {} was not found healthy endpoints, using default config:",
                        name
                    );

                    if transport.is_ok() {
                        for rpc_data in &transport.as_ref().unwrap().rpc {
                            let wc: Result<WebSocketClientUrl, _> =
                                rpc_data.websocket.clone().try_into();
                            if wc.is_ok() {
                                chain.rpc_addr = rpc_data.rpc_address.clone();
                                chain.event_source = EventSourceMode::Push {
                                    url: wc.unwrap(),
                                    batch_delay: default::batch_delay(),
                                };
                                break;
                            }
                        }

                        if &transport.as_ref().unwrap().grpc.len() > &0 {
                            chain.grpc_addr = transport.as_ref().unwrap().grpc[0].clone();
                        }

                        if let EventSourceMode::Push { url, .. } = &chain.event_source {
                            info!("-----------------------------------------------");
                            info!("|  RPC: {}", &chain.rpc_addr);
                            info!("|  GRPC: {}", &chain.grpc_addr);
                            info!("|  Websocket: {}", &url);
                            info!("-----------------------------------------------");
                        } else {
                            info!("-----------------------------------------------");
                            info!("|  RPC: {}", &chain.rpc_addr);
                            info!("|  GRPC: {}", &chain.grpc_addr);
                            info!("|  Websocket: empty");
                            info!("-----------------------------------------------");
                        }
                    }
                }
            }

            endpoints_config.chains.push(endpoints);
        }

        let endpoints_path = home.join("endpoints.json");

        match store_json(&endpoints_config, &endpoints_path) {
            Ok(_) => {
                let msg = format!(
                    "Endpoints config file written successfully : {}.",
                    endpoints_path.to_str().unwrap()
                );
                info!(msg);
                Output::success(msg);
            }
            Err(e) => {
                Output::error(e.to_string());
            }
        }

        match store(&cfg, &path) {
            Ok(_) => {
                warn!("Gas parameters are set to default values.");
                Output::success(format!(
                    "Config file written successfully : {}.",
                    path.to_str().unwrap()
                ))
                .exit()
            }
            Err(e) => Output::error(e.to_string()).exit(),
        }
    }
}
