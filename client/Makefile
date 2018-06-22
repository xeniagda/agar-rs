build:
	cargo +nightly build --release --target wasm32-unknown-unknown
	wasm-bindgen target/wasm32-unknown-unknown/release/agar.wasm --out-dir site
	cp static/* site/
	webpack-cli site/index.js --output site/index.js --mode=production

debug:
	cargo +nightly build --target wasm32-unknown-unknown
	wasm-bindgen target/wasm32-unknown-unknown/debug/agar.wasm --out-dir site
	cp static/* site/
	webpack-cli site/index.js --output site/index.js --mode=development

run:
	open site/index.html
