use substrate_wasm_builder::WasmBuilder;

fn main() {
	// don't run the wasm gen when it is running under rust-analyzer
	// it takes too long, and we don't need it anyway while checking.
	match std::env::var("RUSTC_WRAPPER") {
		Ok(val) if val == "rust-analyzer" => return,
		_ => {},
	}
	WasmBuilder::new()
		.with_current_project()
		.import_memory()
		.export_heap_base()
		.build()
}
