features := cl1_1 cl1_2 cl2 cl2_1 cl2_2 cl3 svm futures

check:
	cargo clippy --no-deps --all-targets --no-default-features -- -Dwarnings
	$(foreach x, $(features), cargo clippy --features $(x);)
	$(foreach x, $(features), cargo clippy --features strict,$(x);)
	cargo clippy --all-features
	cargo clippy --no-default-features --release
	$(foreach x, $(features), cargo clippy --release --features $(x);)
	$(foreach x, $(features), cargo clippy --release --features strict,$(x);)
	cargo clippy --release --all-features

test:
	cargo test --no-default-features
	$(foreach x, $(features), cargo test --features $(x);)
	$(foreach x, $(features), cargo test --features strict,$(x);)
	cargo test --all-features

miri:
	RUST_BACKTRACE=1 MIRIFLAGS="-Zmiri-disable-isolation" cargo miri test --no-default-features
	$(foreach x, $(features), RUST_BACKTRACE=1 MIRIFLAGS="-Zmiri-disable-isolation" cargo miri test --features $(x);)
	$(foreach x, $(features), RUST_BACKTRACE=1 MIRIFLAGS="-Zmiri-disable-isolation" cargo miri test --features strict,$(x);)
	RUST_BACKTRACE=1 MIRIFLAGS="-Zmiri-disable-isolation" cargo miri test --all-features

doc:
	cargo rustdoc --open --all-features -- --cfg docsrs

book:
	cd docs && mdbook serve
	