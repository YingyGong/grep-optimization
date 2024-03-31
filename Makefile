# For test compatibility, your makefile should include 'bin/grep' as a target
# that builds with optimizations on. For C, Makefiles will vary.
# Here is a basic example of how to do this for a rust project:
bin/grep: $(shell find src)
	mkdir -p bin
	cargo build --release
	cp target/release/grep $@

# A 'clean' target is expected as well
clean:
	rm -f bin/*
.PHONY: clean
