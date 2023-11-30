#![allow(clippy::all)]
// Copyright 2023 Webb Technologies Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Signer module to sign data with wdkg keytype in keystore
use sc_cli::{Error, KeystoreParams, SharedParams, SubstrateCli};
use sc_keystore::LocalKeystore;
use sc_service::{config::KeystoreConfig, BasePath};
use sp_keystore::KeystorePtr;
use std::fmt::Debug;

/// The `chain-info` subcommand used to output db meta columns information.
#[derive(Debug, Clone, clap::Parser)]
pub struct DKGSignerCmd {
	/// Key type, examples: "gran", or "imon".
	#[arg(long)]
	data: String,

	#[allow(missing_docs)]
	#[clap(flatten)]
	pub shared_params: SharedParams,

	#[allow(missing_docs)]
	#[clap(flatten)]
	pub keystore_params: KeystoreParams,
}

impl DKGSignerCmd {
	/// Run the command
	pub fn run<C: SubstrateCli>(&self, cli: &C) -> Result<(), Error> {
		let base_path = self
			.shared_params
			.base_path()?
			.unwrap_or_else(|| BasePath::from_project("", "", &C::executable_name()));
		let chain_id = self.shared_params.chain_id(self.shared_params.is_dev());
		let chain_spec = cli.load_spec(&chain_id)?;
		let config_dir = base_path.config_dir(chain_spec.id());

		let keystore = match self.keystore_params.keystore_config(&config_dir)? {
			KeystoreConfig::Path { path, password } => {
				let keystore: KeystorePtr = LocalKeystore::open(path, password)?.into();
				keystore
			},
			_ => unreachable!("keystore_config always returns path and password; qed"),
		};

		let key_type = dkg_runtime_primitives::KEY_TYPE;

		let maybe_public = keystore.ecdsa_public_keys(key_type);
		let public = maybe_public.first().unwrap();

		let signature = keystore
			.ecdsa_sign(key_type, &public, &self.data.as_bytes())
			.map_err(|_| Error::KeystoreOperation)?;

		println!("Signature : {:?}", signature);

		Ok(())
	}
}
