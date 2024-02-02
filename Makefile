DEBUG_TARGET := target/target/debug/kfs
TARGET := $(DEBUG_TARGET)

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
	cargo build

.PHONY: run
run:
	cargo build
	qemu-system-i386 -kernel $(TARGET)

.PHONY: clean
clean:
	cargo clean

.PHONY: re
re:
	@make --no-print-directory clean
	@make --no-print-directory build
