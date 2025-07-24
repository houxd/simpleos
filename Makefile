

all: no_std_buddy_alloc task_emulate allocator_test


no_std_buddy_alloc:
	cargo build --target thumbv7em-none-eabi --example no_std_buddy_alloc

task_emulate:
	cargo build --example task_emulate

allocator_test:
	cargo build --target thumbv7em-none-eabi --example allocator_test --features="allocator panic-handler"