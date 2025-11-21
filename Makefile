
FEATURES_ARGS = \
	--no-default-features \
	--features="util"

all: 
	cargo build --examples $(FEATURES_ARGS)

emulate:
	cargo run --example emulate $(FEATURES_ARGS)

clean:
	cargo clean