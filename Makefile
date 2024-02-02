DEBUG_TARGET := target/target/debug/kfs
RELEASE_TARGET := target/target/release/kfs
TARGET := $(DEBUG_TARGET)

CARGO_FLAGS :=

ifeq ($(RELEASE), 1)
	TARGET := $(RELEASE_TARGET)
	CARGO_FLAGS += --release
endif

.PHONY: help
help:
	@echo "available commands:"
	@echo "  make help"
	@echo "  make build"
	@echo "  make run"
	@echo "  make clean"
	@echo "  make re"

.PHONY: build
build:
	cargo build $(CARGO_FLAGS)

.PHONY: run
run:
	cargo build $(CARGO_FLAGS)
	qemu-system-i386 -kernel $(TARGET) -machine type=pc-i440fx-3.1

.PHONY: clean
clean:
	cargo clean

.PHONY: re
re:
	@make --no-print-directory clean
	@make --no-print-directory build
