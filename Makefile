DEBUG_TARGET := target/target/debug/kfs
RELEASE_TARGET := target/target/release/kfs
TARGET := $(DEBUG_TARGET)

QEMU_FLAGS := -machine type=pc-i440fx-3.1 -m 2G -serial stdio
CARGO_FLAGS :=

ifeq ($(RELEASE), 1)
	TARGET := $(RELEASE_TARGET)
	CARGO_FLAGS := $(CARGO_FLAGS) --release
endif

.PHONY: help
help:
	@echo "available commands:"
	@echo "  make help          print this message"
	@echo "  make build         build the kernel"
	@echo "  make run           run the kernel with QEMU"
	@echo "  make print-size    print the size of the kernel"
	@echo "  make clean         remove intermediate files"
	@echo "  make re            clean then build the kernel again"

.PHONY: build
build:
	cargo build $(CARGO_FLAGS)

.PHONY: run
run:
	cargo build $(CARGO_FLAGS)
	qemu-system-i386 -kernel $(TARGET) $(QEMU_FLAGS)

.PHONY: print-size
print-size:
	@cargo build -q $(CARGO_FLAGS)
	@du -h $(TARGET)

.PHONY: clean
clean:
	cargo clean

.PHONY: re
re:
	@make --no-print-directory clean
	@make --no-print-directory build
