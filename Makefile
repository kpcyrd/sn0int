check:
	(cd sn0int-registry/sn0int-common; cargo check)
	(cd sn0int-registry; cargo check)
	cargo check

force-check:
	(cd sn0int-registry/sn0int-common; touch src/lib.rs; cargo check)
	(cd sn0int-registry; touch src/main.rs; cargo check)
	touch src/lib.rs
	cargo check

test:
	(cd sn0int-registry/sn0int-common; cargo test)
	(cd sn0int-registry; cargo test)
	cargo test
	cargo test -- --ignored

update:
	get-oui -v -u http://standards-oui.ieee.org/oui/oui.txt -f data/ieee-oui.txt
	get-iab -v -u http://standards-oui.ieee.org/iab/iab.txt -f data/ieee-iab.txt
	rm -f data/ieee-*.txt.bak
