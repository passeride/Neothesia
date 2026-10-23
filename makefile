build-app:
	cargo build --release --bin tangent
run-app:
	cargo run --release --bin tangent

install-app:
	cargo install --path tangent

check-recorder:
	cargo check -p tangent-cli
build-recorder:
	cargo build --release -p tangent-cli
run-recorder:
	cargo run --release -p tangent-cli -- $(file)

