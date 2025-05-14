/* call_analyze.c
 *
 * Example of loading your Rust‐compiled shared library at runtime
 * and invoking `analyze_network_dir(const char*, int)`.
 *
 * Compile:
 *   gcc -o call_analyze call_analyze.c -ldl
 *
 * Usage:
 *   ./call_analyze ./sample_data 10
 */

#include <stdio.h>
#include <stdlib.h>

extern int analyze_network_dir(const char* dir_path, int top_n);

int main(int argc, char *argv[]) {
    if (argc != 3) {
        fprintf(stderr, "Usage: %s <csv_dir> <top_n>\n", argv[0]);
        return EXIT_FAILURE;
    }

    const char *csv_dir = argv[1];
    int top_n = atoi(argv[2]);

    int result = analyze_network_dir(csv_dir, top_n);
    if (result != 0) {
        fprintf(stderr, "analyze_network_dir returned error code %d\n", result);
    } else {
        printf("✅ Parquet file generated under \"%s\"\n", csv_dir);
    }

    return result;
}
