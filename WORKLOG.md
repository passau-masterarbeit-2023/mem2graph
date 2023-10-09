# Work logs

* [ ] Create new embedding: embed just the first block of a chunk. Optionally, add a filtering based on entropy.
* [ ] Finish entropy filtering. Fix, since some keys are missing.
* [ ] Test the pipelines and debug

### Mon 9 Oct 2023


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
