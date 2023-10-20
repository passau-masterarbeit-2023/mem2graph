# Work logs

### Fri 20 Oct 2023

* [X] Added filtering based embedding field to `graph-with-embedding-comment` pipeline.
* [X] Modified script launcher to support timeout and multiple pipeline selection.

### Sun 15 Oct 2023

* [X] Fixed error with output file naming. It was keeping the filepath of the input, and not the real one from files.

### Tue 10 Oct 2023

List of commands

```shell
cargo run -- -d /home/onyr/code/phdtrack/phdtrack_data_clean -o /home/onyr/code/phdtrack/mem2graph/data/memory_graphs -p graph
cargo run -- -d /home/onyr/code/phdtrack/phdtrack_data_clean -o /home/onyr/code/phdtrack/mem2graph/data/memory_graphs_no_vn_chn_annotations -p graph -v -a chunk-header-node
cargo run -- -d /home/onyr/code/phdtrack/phdtrack_data_clean -o /home/onyr/code/phdtrack/mem2graph/data/chunk_semantic_embedding_no_vn_max_chunk -p chunk-semantic-embedding -e only-max-entropy -v -a chunk-header-node
cargo run -- -d /home/onyr/code/phdtrack/phdtrack_data_clean -o /home/onyr/code/phdtrack/mem2graph/data/chunk_semantic_embedding_no_vn_max_chunk -p chunk-semantic-embedding -e only-max-entropy -v -a chunk-header-node
cargo run -- -d /home/onyr/code/phdtrack/phdtrack_data_clean -o /home/onyr/code/phdtrack/mem2graph/data/chunk_semantic_embedding_no_vn_threshold_entropy -p chunk-semantic-embedding -e min-of-chunk-treshold-entropy -v -a chunk-header-node
cargo run -- -d /home/onyr/code/phdtrack/phdtrack_data_clean -o /home/onyr/code/phdtrack/mem2graph/data/chunk_statistic_embedding -p chunk-statistic-embedding -a chunk-header-node
cargo run -- -d /home/onyr/code/phdtrack/phdtrack_data_clean -o /home/onyr/code/phdtrack/mem2graph/data/chunk_statistic_embedding_max_entropy -p chunk-statistic-embedding -a chunk-header-node -e only-max-entropy

```

Fixed memory graph to image python script.

Fixed rust test.

* [X] Finish entropy filtering. Fix, since some keys are missing. >NEED TESTING
* [X] Test the pipelines and debug
* [X] Factorise CSV header (static list of factory function). >NEED TESTING
* [X] Add in chunk semantic embedding, and chunk top vn embedding, the chunk number in the heap dump (0, 1, 2, 3...)
* [X] Added script to launch all pipelines with necessary cli arguments.

### Mon 9 Oct 2023

Added new pipeline `chunk_top_vn_semantic_embedding`.

* [X] Create new embedding: embed just the first block of a chunk. Optionally, add a filtering based on entropy.

### Fri 4 Oct 2023

* [X] Split the file of embedding.
* [X] filtering by entropy

### Tue 3 Oct 2023

* [X] Refactoring: Put KeyData outside of Node. !!!
* [X] Refactoring multi-annotation-embedding

### Mon 2 Oct 2023

Refactored `chunk_step` and check it was correct. Added new `FooterNode` integration. Fixed the SSH_STRUCT warning which was misleading and wrong.

Note that if the last chunk is incomplete, we skip it.

### Fri 22 Sep 2023

```shell
cargo run -- -p graph -f /home/onyr/code/phdtrack/phdtrack_data_clean/Training/Training/basic/V_7_1_P1/24/17016-1643962152-heap.raw -o /home/onyr/code/phdtrack/mem2graph/graphs/graphs

cargo run -- -p graph -f /home/onyr/code/phdtrack/phdtrack_data_clean/Training/Training/scp/V_7_8_P1/16/302-1644391327-heap.raw -o /home/onyr/code/phdtrack/mem2graph/graphs/graphs
```
