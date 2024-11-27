//! `config` subcommand

use abscissa_core::clap::Parser;
use abscissa_core::{Command, Runnable};

mod auto;
mod validate;
mod endpointsvalidate;

/// `config` subcommand
#[derive(Command, Debug, Parser, Runnable)]
pub enum ConfigCmd {
    /// Validate the relayer configuration
    Validate(validate::ValidateCmd),

    /// Automatically generate a config.toml for the specified chain(s)
    Auto(auto::AutoCmd),

    /// Update endpoints in the configuration file and replace it for healthy ones.
    Endpoints(endpointsvalidate::HealthyEndpointsCmd)
}
