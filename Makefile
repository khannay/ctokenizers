# Top-level Makefile

# Directories
RUST_DIR := rust_tokenizer
RUST_TARGET := $(RUST_DIR)/target/release
RUST_LIB := $(RUST_TARGET)/libbpe_tokenizer_ffi.so
C_SRC := main.c
C_BIN := myapp

# Compiler and flags
CC := gcc
CFLAGS := -Wall -Werror
LDFLAGS := -L$(RUST_TARGET) -lbpe_tokenizer_ffi -ldl -lpthread

.PHONY: all clean

all: $(C_BIN)

# Build Rust dynamic lib
$(RUST_LIB):
	cargo build --release --manifest-path $(RUST_DIR)/Cargo.toml

# Build C executable
$(C_BIN): $(C_SRC) $(RUST_LIB)
	$(CC) $(CFLAGS) -o $@ $(C_SRC) $(LDFLAGS)

clean:
	cargo clean --manifest-path $(RUST_DIR)/Cargo.toml
	rm -f $(C_BIN)

