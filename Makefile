check:
	(cd sn0int-common; cargo check)
	(cd sn0int-registry; cargo check)
	cargo check

force-check:
	(cd sn0int-common; touch src/lib.rs; cargo check)
	(cd sn0int-registry; touch src/main.rs; cargo check)
	touch src/lib.rs
	cargo check

test:
	(cd sn0int-common; cargo test)
	(cd sn0int-registry; cargo test)
	cargo test
	cargo test -- --ignored
