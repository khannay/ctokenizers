
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <dirent.h>
#include <stdint.h>
#include "uthash.h"

#define MAX_LINE 1024
#define MAX_IP_LEN 16
#define MAX_PROTO_LEN 16

typedef struct {
    char src_ip[MAX_IP_LEN];
    int src_port;
    char dst_ip[MAX_IP_LEN];
    int dst_port;
    char protocol[MAX_PROTO_LEN];
    int label;
} FlowKey;

typedef struct {
    FlowKey key;
    int count;
    UT_hash_handle hh;
} CountEntry;

CountEntry *counter = NULL;

void insert_or_increment(FlowKey *k) {
    CountEntry *entry;
    HASH_FIND(hh, counter, k, sizeof(FlowKey), entry);
    if (entry) {
        entry->count += 1;
    } else {
        entry = malloc(sizeof(CountEntry));
        memcpy(&entry->key, k, sizeof(FlowKey));
        entry->count = 1;
        HASH_ADD(hh, counter, key, sizeof(FlowKey), entry);
    }
}

int parse_csv_file(const char* path) {
    FILE *fp = fopen(path, "r");
    if (!fp) return -1;

    char line[MAX_LINE];
    fgets(line, MAX_LINE, fp); // skip header

    while (fgets(line, MAX_LINE, fp)) {
        FlowKey k;
        char *token = strtok(line, ",");
        if (!token) continue;
        strncpy(k.src_ip, token, MAX_IP_LEN);

        token = strtok(NULL, ",");
        k.src_port = atoi(token ? token : "0");

        token = strtok(NULL, ",");
        strncpy(k.dst_ip, token ? token : "", MAX_IP_LEN);

        token = strtok(NULL, ",");
        k.dst_port = atoi(token ? token : "0");

        token = strtok(NULL, ",");
        strncpy(k.protocol, token ? token : "", MAX_PROTO_LEN);

        token = strtok(NULL, ",");
        k.label = atoi(token ? token : "0");

        insert_or_increment(&k);
    }

    fclose(fp);
    return 0;
}

typedef struct {
    FlowKey key;
    int count;
} ResultEntry;

int compare_desc(const void *a, const void *b) {
    return ((ResultEntry*)b)->count - ((ResultEntry*)a)->count;
}

int main(int argc, char** argv) {
    if (argc < 3) {
        fprintf(stderr, "Usage: %s <csv_dir> <top_n>\n", argv[0]);
        return 1;
    }

    const char* dirpath = argv[1];
    int top_n = atoi(argv[2]);

    DIR *d = opendir(dirpath);
    if (!d) {
        perror("opendir");
        return 2;
    }

    struct dirent *entry;
    while ((entry = readdir(d))) {
        if (strstr(entry->d_name, ".csv")) {
            char fullpath[512];
            snprintf(fullpath, sizeof(fullpath), "%s/%s", dirpath, entry->d_name);
            parse_csv_file(fullpath);
        }
    }
    closedir(d);

    int total = HASH_COUNT(counter);
    ResultEntry *results = malloc(sizeof(ResultEntry) * total);
    CountEntry *e, *tmp;
    int idx = 0;

    HASH_ITER(hh, counter, e, tmp) {
        results[idx].key = e->key;
        results[idx].count = e->count;
        idx++;
        free(e);
    }
    HASH_CLEAR(hh, counter);

    qsort(results, total, sizeof(ResultEntry), compare_desc);
    printf("Top %d flows:\n", top_n);
    for (int i = 0; i < top_n && i < total; ++i) {
        printf("%s:%d -> %s:%d [%s] label=%d count=%d\n",
            results[i].key.src_ip, results[i].key.src_port,
            results[i].key.dst_ip, results[i].key.dst_port,
            results[i].key.protocol, results[i].key.label,
            results[i].count);
    }

    free(results);
    return 0;
}
