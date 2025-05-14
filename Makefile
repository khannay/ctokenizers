# Top-level Makefile

# Directories & filenames
RUST_DIR     := rust_tokenizer
RUST_TARGET  := $(RUST_DIR)/target/release
RUST_LIB     := $(RUST_TARGET)/libbpe_tokenizer_ffi.so

# First C program (your existing app)
C_SRC        := main.c
C_BIN        := main.exe

# Second C program (the "call_analyze" example)
CALL_SRC     := call_analyze.c
CALL_BIN     := call_analyze.exe

# Compiler and linker flags
CC           := gcc
CFLAGS       := -Wall -Werror
LDFLAGS      := -L$(RUST_TARGET) -lbpe_tokenizer_ffi -ldl -lpthread

.PHONY: all clean

all: $(C_BIN) $(CALL_BIN) $(RUST_LIB)

# 1) Build the Rust shared library
$(RUST_LIB):
	cargo build --release --manifest-path $(RUST_DIR)/Cargo.toml

# 2) Build your existing C binary
$(C_BIN): $(C_SRC) $(RUST_LIB)
	$(CC) $(CFLAGS) -o $@ $< $(LDFLAGS)

# 3) Build the new test_program from call_analyze.c
$(CALL_BIN): $(CALL_SRC) $(RUST_LIB)
	$(CC) $(CFLAGS) -o $@ $< $(LDFLAGS)

clean:
	cargo clean --manifest-path $(RUST_DIR)/Cargo.toml
	rm -f $(C_BIN) $(CALL_BIN)
