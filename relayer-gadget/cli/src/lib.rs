use std::path::PathBuf;

/// Cli tool to interact with Webb Relayer CLI
#[derive(Debug, Clone, clap::Parser)]
#[clap(next_help_heading = "Webb Relayer")]
pub struct WebbRelayerCmd {
	/// Directory that contains configration files for the relayer.
	#[arg(long, value_name = "PATH")]
	pub relayer_config_dir: Option<PathBuf>,
}
