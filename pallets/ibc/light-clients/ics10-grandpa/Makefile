module-grandpa=ics10-grandpa

check-ics10-grandpa:
	cargo +nightly check -p $(module-grandpa) --no-default-features --target=wasm32-unknown-unknown

build-ics10-grandpa:
	cargo +nightly build -p $(module-grandpa) --no-default-features --target=wasm32-unknown-unknown

test-ics10-grandpa:
	cargo +nightly t -p $(module-grandpa)