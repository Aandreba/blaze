features := cl1_1 cl1_2 cl2 cl2_1 cl2_2 cl3 svm futures

check:
	cargo clippy --no-deps --all-targets -- -Dwarnings
	cargo clippy --no-deps --all-targets --features cl3,futures -- -Dwarnings
	cargo +nightly clippy --no-deps --all-targets --all-features -- -Dwarnings

doc:
	cargo rustdoc --open --all-features -- --cfg docsrs

book:
	cd docs && mdbook serve
	