features := cl1_1 cl1_2 cl2 cl2_1 cl3 svm futures strict

check:
	cargo check --no-default-features
	$(foreach x, $(features), cargo check --features $(x);)
	cargo check --all-features

doc:
	cargo rustdoc --open --all-features -- --cfg docsrs