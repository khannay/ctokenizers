#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>

typedef struct Tokenizer Tokenizer;

// FFI functions exposed from Rust
extern Tokenizer* load_tokenizer(const char* path);
extern uint32_t* encode_text(Tokenizer* tokenizer, const char* input, size_t* out_len);
extern void free_tokenizer(Tokenizer* tokenizer);
extern void free_encoded(uint32_t* ids, size_t len);
extern Tokenizer* load_tokenizer(const char* path);
extern uint32_t** encode_batch(Tokenizer* tokenizer, const char** inputs, size_t num_inputs, size_t* out_lengths);
extern void free_encoded_batch(uint32_t** token_ids, const size_t* lengths, size_t count);

extern int analyze_network_dir(const char* dir_path, int top_n);

int main() {
    const char* tokenizer_path = "tokenizer.json";
    Tokenizer* tokenizer = load_tokenizer(tokenizer_path);
    if (!tokenizer) {
        fprintf(stderr, "Failed to load tokenizer from %s\n", tokenizer_path);
        return 1;
    }

    const char* text = "Hello, world! This is a test.";
    size_t token_count = 0;
    uint32_t* ids = encode_text(tokenizer, text, &token_count);

    if (!ids) {
        fprintf(stderr, "Encoding failed.\n");
        free_tokenizer(tokenizer);
        return 1;
    }

    printf("Encoded token IDs (%zu):\n", token_count);
    for (size_t i = 0; i < token_count; ++i) {
        printf("%u ", ids[i]);
    }
    printf("\n");

    const char* texts[] = {
        "Hello world",
        "This is the second input",
        "Third input string for batching"
    };
    size_t num_inputs = 3;
    size_t* lengths = malloc(sizeof(size_t) * num_inputs);
    
    uint32_t** results = encode_batch(tokenizer, texts, num_inputs, lengths);
    
    for (size_t i = 0; i < num_inputs; ++i) {
        printf("Input %zu tokens (%zu): ", i, lengths[i]);
        for (size_t j = 0; j < lengths[i]; ++j) {
            printf("%u ", results[i][j]);
        }
        printf("\n");
    }
    
    free_encoded_batch(results, lengths, num_inputs);
    free(lengths);
    return 0;    

    free_encoded(ids, token_count);
    free_tokenizer(tokenizer);
    return 0;
}

