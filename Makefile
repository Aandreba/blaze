features := cl1_1 cl1_2 cl2 cl2_1 cl2_2 cl3 svm futures strict

check:
	cargo check --no-default-features
	$(foreach x, $(features), cargo check --features $(x);)
	cargo check --all-features

test:
	cargo test --no-default-features
	$(foreach x, $(features), cargo test --features $(x);)
	cargo test --all-features

doc:
	cargo rustdoc --open --all-features -- --cfg docsrs