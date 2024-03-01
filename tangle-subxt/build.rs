use std::error::Error;
use subxt_codegen::TypeSubstitutes;

mod substrate {
	use super::*;
	use parity_scale_codec::Decode;
	use subxt_codegen::CratePath;

	fn parse_and_generate_runtime(path: &str, out: &str) -> Result<(), Box<dyn Error>> {
		println!("cargo:rerun-if-changed=./{}", path);
		let bytes = std::fs::read(path)?;

		let metadata = <subxt_metadata::Metadata as Decode>::decode(&mut &bytes[..])?;
		let crate_path = CratePath::default();
		// Module under which the API is generated.
		let item_mod = syn::parse_quote!(
			pub mod api {}
		);
		// Default type substitutes.
		let substs = TypeSubstitutes::with_default_substitutes(&crate_path);
		// Generate the Runtime API.
		let generator = subxt_codegen::RuntimeGenerator::new(metadata);
		let mut generated_type_derives =
			subxt_codegen::DerivesRegistry::with_default_derives(&crate_path);

		generated_type_derives.extend_for_all(
			[syn::parse_quote!(Eq), syn::parse_quote!(PartialEq), syn::parse_quote!(Clone)],
			[],
		);

		// Include metadata documentation in the Runtime API.
		let generate_docs = true;
		let runtime_api = generator.generate_runtime(
			item_mod,
			generated_type_derives,
			substs,
			crate_path,
			generate_docs,
		)?;
		let syntax_tree = syn::parse_file(&runtime_api.to_string()).unwrap();
		let formatted = prettyplease::unparse(&syntax_tree);
		std::fs::write(out, formatted)?;
		Ok(())
	}

	pub fn generate_tangle_runtime() -> Result<(), Box<dyn Error>> {
		parse_and_generate_runtime("metadata/tangle-runtime.scale", "src/tangle_runtime.rs")
	}
}

fn main() -> Result<(), Box<dyn Error>> {
	{
		substrate::generate_tangle_runtime()?;
	}
	Ok(())
}
