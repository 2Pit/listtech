# Listtech Search Platform

A Rust-based search platform for document indexing and full-text + faceted search, with versioned schema support.

## Quick Start

### 1. Download the Amazon Electronics dataset

Download the dataset from Kaggle or the UCSD website:
https://cseweb.ucsd.edu/~jmcauley/datasets/amazon_v2/
(For example, use the **Electronics** metadata)

Place the file `meta_Electronics.json` into the `./data/` directory:

```bash
project-root/
|
├── data/
|   └── meta_Electronics.json
```

### 2. Run the indexer and searcher

```bash
cargo run -p indexer
cargo run -p searcher --release
```

### 3. Register a schema and ingest documents

```bash
make add_schema
make ingest
```

This will:

- Register the active schema
- Initialize the index directory
- Send documents from `meta_Electronics.json` to the indexer

### 4. Run a sample query

```bash
make select
```

This performs a search request from script via HTTP using JSON (CBOR is also supported).

## Project Structure

- `data/` – input JSON documents and generated search indexes
- `indexer/` – indexing component (schema registration, ingestion, auto-commits)
- `searcher/` – search component (query parsing, facets, sorting)
- `scripts/` – helper scripts for schema setup and ingestion

## Requirements

- Rust 1.76 or later
- GNU Make

## License

This project is licensed under the **Business Source License 1.1 (BSL-1.1)**.

- **Commercial use is not permitted** until the change date.
- On **June 1, 2029**, the license will automatically convert to **Apache License 2.0**.
- You may use, modify, and share this code for **non-commercial purposes** until then.
- To use this software in a commercial or production environment before the change date, a separate license must be obtained from the author.

See the [LICENSE.md](./LICENSE.md) file for full terms.

Third-party components and their licenses are listed in [third_party_licenses.html](./third_party_licenses.html).
