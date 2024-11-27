//! Contains traits to query nodes of a given chain from their APIs.
//! Contains struct to perform a health check on a gRPC/WebSocket endpoint and
//! to retrieve the `max_block_size` from a RPC endpoint.

use std::fmt::Debug;
use std::str::FromStr;
use async_trait::async_trait;
use futures::{stream::FuturesUnordered, StreamExt};
use futures::future::{select, FutureExt};
use std::error::Error;
use tokio::time::{Duration, sleep};
use http::Uri;
use tokio::time::timeout;
use tracing::{info, warn};
use serde::{Deserialize, Serialize};

use ibc_proto::cosmos::bank::v1beta1::query_client::QueryClient;
use tendermint_rpc::{Client, SubscriptionClient, Url, WebSocketClient};

use crate::error::RegistryError;
use crate::formatter::{SimpleWebSocketFormatter, UriFormatter};

const WEBSOCKET_TIMEOUT_SECS: u64 = 5;
const GRPC_TIMEOUT_SECS: u64 = 20;

/// `QueryTypes` represents the basic types required to query a node
pub trait QueryTypes {
    /// `QueryInput` represents the data needed to query a node. It is typically a URL
    type QueryInput: Debug + Send;
    /// `QueryOutput` represents the data returned by your query
    type QueryOutput: Debug + Send + Sync;
    /// `QueryOutput` represents the error returned when a query fails
    type QueryError: Error + Debug + Send + Sync;
}

#[async_trait]
/// `QueryContext` represents the basic expectations for a query
pub trait QueryContext: QueryTypes {
    /// Return an error specific to the query which is returned when `query_healthy` fails
    ///
    /// # Arguments
    ///
    /// * `chain_name` - A string slice that holds the name of a chain
    fn query_error(chain_name: String) -> Self::QueryError;

    /// Query an endpoint and return the result
    ///
    /// # Arguments
    ///
    /// * `url` - A `QueryInput` object that holds the data needed to query a node
    async fn query(url: Self::QueryInput) -> Result<Self::QueryOutput, Self::QueryError>;

    /// Query every endpoint from a list of urls and return the output of the first one to answer.
    ///
    /// # Arguments
    ///
    /// * `chain_name` - A string that holds the name of a chain
    /// * `urls` - A vector of urls to query
    async fn query_healthy(
        chain_name: String,
        urls: Vec<Self::QueryInput>,
    ) -> Result<Self::QueryOutput, Self::QueryError> {
        info!("Trying to find a healthy RPC endpoint for chain {chain_name}");
        info!("Trying the following RPC endpoints: {urls:?}");

        let mut futures: FuturesUnordered<_> =
            urls.into_iter()
                .map(|url| Self::query(url))
                .collect();

        while let Some(result) = futures.next().await {
            if result.is_ok() {
                return result;
            }
        }

        Err(Self::query_error(chain_name))
    }

    /// Query every endpoint from a list of urls and return the output of the first one to answer.
    ///
    /// # Arguments
    ///
    /// * `chain_name` - A string that holds the name of a chain
    /// * `urls` - A vector of urls to query
    async fn query_healthy_all(
        chain_name: String,
        urls: Vec<Self::QueryInput>,
    ) -> Result<Vec<Self::QueryOutput>, Self::QueryError> {

        println!("Trying to find a healthy RPC endpoint for chain {chain_name}");

        let mut fut: FuturesUnordered<_> =
            urls.into_iter()
                .map(|url| Self::query(url))
                .collect();

        let mut res: Vec<Self::QueryOutput> = Vec::new();

        while let Some(result) = fut.next().await {
            let mapped = result
                .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>);

            let left = async{mapped}.boxed();
            let right = sleep(Duration::from_secs(20)).boxed();

            match select (left, right).await {
                futures::future::Either::Left((result, _)) => {
                    if result.is_ok() {
                        res.push(result.unwrap());
                    }
                }
                futures::future::Either::Right((_, _)) => {
                    warn!("Timeout reached");
                }
            }
        }

        if res.len() > 0 {
            return Ok(res);
        }

        Err(Self::query_error(chain_name))
    }
}

// ----------------- VOID RPC ------------------

pub struct VoidHermesRpcQuerier;

/// Expected Input, Output and Error to query an RPC endpoint
impl QueryTypes for VoidHermesRpcQuerier {
    type QueryInput = String;
    type QueryOutput = HermesConfigData;
    type QueryError = RegistryError;
}

#[async_trait]
impl QueryContext for VoidHermesRpcQuerier {
    fn query_error(chain_name: String) -> RegistryError {
        RegistryError::no_healthy_rpc(chain_name)
    }

    /// Convert the RPC url to a WebSocket url, query the endpoint, return the data from the RPC.
    async fn query(rpc: Self::QueryInput) -> Result<Self::QueryOutput, Self::QueryError> {
        let websocket_addr = SimpleWebSocketFormatter::parse_or_build_address(rpc.as_str())?;
        Ok(HermesConfigData {
            rpc_address: Url::from_str(&rpc)
                .map_err(|e| RegistryError::tendermint_url_parse_error(rpc, e))?,
            max_block_size: 0,
            websocket: websocket_addr,
        })
    }
}

// ------------------ Combined RPC and WebSocket -------------

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TransportConfigData{
    pub rpc: Vec<HermesConfigData>,
    pub grpc: Vec<Url>,
}

// ----------------- SimpleHermesRpcQuerier ------------------

/// `SimpleHermesRpcQuerier` retrieves `HermesConfigData` by querying a list of RPC endpoints through their WebSocket API
/// and returns the result of the first endpoint to answer.
pub struct SimpleHermesRpcQuerier;

/// Data which must be retrieved from RPC endpoints for Hermes
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct HermesConfigData {
    pub rpc_address: Url,
    #[serde(skip)]
    pub max_block_size: u64,
    pub websocket: Url,
    // max_block_time should also be retrieved from the RPC
    // however it looks like it is not in the genesis file anymore
}

/// Expected Input, Output and Error to query an RPC endpoint
impl QueryTypes for SimpleHermesRpcQuerier {
    type QueryInput = String;
    type QueryOutput = HermesConfigData;
    type QueryError = RegistryError;
}

#[async_trait]
impl QueryContext for SimpleHermesRpcQuerier {
    /// Return an error `NoHealthyRpc` when `query_healthy` fails
    fn query_error(chain_name: String) -> RegistryError {
        RegistryError::no_healthy_rpc(chain_name)
    }

    /// Convert the RPC url to a WebSocket url, query the endpoint, return the data from the RPC.
    async fn query(rpc: Self::QueryInput) -> Result<Self::QueryOutput, Self::QueryError> {
        let websocket_addr = SimpleWebSocketFormatter::parse_or_build_address(rpc.as_str())?;

        let (client, driver) = timeout(
            Duration::from_secs(WEBSOCKET_TIMEOUT_SECS),
            WebSocketClient::new(websocket_addr.clone()),
        )
        .await
        .map_err(|e| RegistryError::websocket_time_out_error(websocket_addr.to_string(), e))?
        .map_err(|e| RegistryError::websocket_connect_error(websocket_addr.to_string(), e))?;

        let driver_handle = tokio::spawn(async move { driver.run().await });

        let latest_consensus_params = match client.latest_consensus_params().await {
            Ok(response) => response.consensus_params.block.max_bytes,
            Err(e) => {
                return Err(RegistryError::rpc_consensus_params_error(
                    websocket_addr.to_string(),
                    e,
                ))
            }
        };

        client.close().map_err(|e| {
            RegistryError::websocket_conn_close_error(websocket_addr.to_string(), e)
        })?;

        driver_handle
            .await
            .map_err(|e| RegistryError::join_error("chain_data_join".to_string(), e))?
            .map_err(|e| RegistryError::websocket_driver_error(websocket_addr.to_string(), e))?;

        Ok(HermesConfigData {
            rpc_address: Url::from_str(&rpc)
                .map_err(|e| RegistryError::tendermint_url_parse_error(rpc, e))?,
            max_block_size: latest_consensus_params,
            websocket: websocket_addr,
        })
    }
}

// ----------------- VOID GRPC ------------------
pub struct VoidGrpcHealthCheckQuerier;
/// Expected Input and Output to query a GRPC endpoint
impl QueryTypes for VoidGrpcHealthCheckQuerier {
    type QueryInput = Uri;
    type QueryOutput = Url;
    type QueryError = RegistryError;
}

#[async_trait]
impl QueryContext for VoidGrpcHealthCheckQuerier {
    /// Return an error `NoHealthyGrpc` when `query_healthy` fails
    fn query_error(chain_name: String) -> Self::QueryError {
        RegistryError::no_healthy_grpc(chain_name)
    }

    async fn query(uri: Self::QueryInput) -> Result<Self::QueryOutput, Self::QueryError> {
        let tendermint_url = uri
            .to_string()
            .parse()
            .map_err(|e| RegistryError::tendermint_url_parse_error(uri.to_string(), e))?;

        Ok(tendermint_url)
    }
}

// ----------------- GRPC ------------------

/// `GrpcHealthCheckQuerier` connects to a list of gRPC endpoints
/// and returns the URL of the first one to answer.
pub struct GrpcHealthCheckQuerier;

/// Expected Input and Output to query a GRPC endpoint
impl QueryTypes for GrpcHealthCheckQuerier {
    type QueryInput = Uri;
    type QueryOutput = Url;
    type QueryError = RegistryError;
}

#[async_trait]
impl QueryContext for GrpcHealthCheckQuerier {
    /// Return an error `NoHealthyGrpc` when `query_healthy` fails
    fn query_error(chain_name: String) -> Self::QueryError {
        RegistryError::no_healthy_grpc(chain_name)
    }

    /// Query the endpoint and return the GRPC url
    async fn query(uri: Self::QueryInput) -> Result<Self::QueryOutput, Self::QueryError> {
        let tendermint_url = uri
            .to_string()
            .parse()
            .map_err(|e| RegistryError::tendermint_url_parse_error(uri.to_string(), e))?;

        info!("Querying gRPC server at {tendermint_url}");

        timeout(
            Duration::from_secs(GRPC_TIMEOUT_SECS),
            QueryClient::connect(uri.clone()),
        )
        .await
        .map_err(|e| RegistryError::grpc_time_out_error(uri.to_string(), e))?
        .map_err(|_| RegistryError::unable_to_connect_with_grpc())?;

        // QueryClient::connect(uri)
        //     .await
        //     .map_err(|_| RegistryError::unable_to_connect_with_grpc())?;

        Ok(tendermint_url)
    }
}
