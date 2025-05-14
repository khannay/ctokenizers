# generate_csvs.py

import os
import csv
import random

def generate_test_csvs(directory, num_files=3, rows_per_file=100):
    """
    Creates <directory> (if needed) and writes `num_files` CSVs each
    with `rows_per_file` random network‐flow rows.
    """
    os.makedirs(directory, exist_ok=True)
    protocols = ["TCP", "UDP", "ICMP"]
    labels    = ["OK", "SUSPICIOUS", "MALWARE"]

    for i in range(num_files):
        fname = os.path.join(directory, f"test_{i+1}.csv")
        with open(fname, "w", newline="") as f:
            w = csv.writer(f)
            # header
            w.writerow([
                "source_ip",
                "source_port",
                "dest_ip",
                "dest_port",
                "protocol",
                "label"
            ])
            for _ in range(rows_per_file):
                sip   = f"192.168.{random.randint(0,255)}.{random.randint(1,254)}"
                sport = random.randint(1024, 65535)
                dip   = f"10.0.{random.randint(0,255)}.{random.randint(1,254)}"
                dport = random.randint(1024, 65535)
                proto = random.choice(protocols)
                lab   = random.choice(labels)
                w.writerow([sip, sport, dip, dport, proto, lab])

if __name__ == "__main__":
    # Generates 5 files, each with 200 rows, under ./sample_data/
    generate_test_csvs("sample_data", num_files=5, rows_per_file=200)
    print("✅ CSVs written to ./sample_data/")
