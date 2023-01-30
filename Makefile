features := cl1_1 cl1_2 cl2 cl2_1 cl2_2 cl3 svm futures

check:
	cargo check --no-default-features
	$(foreach x, $(features), cargo check --features $(x);)
	$(foreach x, $(features), cargo check --features strict,$(x);)
	cargo check --all-features
	cargo check --no-default-features --release
	$(foreach x, $(features), cargo check --release --features $(x);)
	$(foreach x, $(features), cargo check --release --features strict,$(x);)
	cargo check --release --all-features

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