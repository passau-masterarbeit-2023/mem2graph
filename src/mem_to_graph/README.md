# Mem to Graph

## run

`cargo run -- -h`: display help on program arguments, display available command flags.

`cargo run`: run program with default values (especially default dir from `.env` config file).

## program params

### `COMPRESS_POINTER_CHAINS`

##### graph pointer to pointer compression

`COMPRESS_POINTER_CHAINS`: bool. Activate or not the pointer compression. This actually means that chains of pointers (i.e. pointers that points to pointers) are compressed in a single edge, where the weight depends on the number of compressed PTR2PTR edges.

> When the compression is active, we no more have links between 2 pointers, and thus, we lose the connetions between different data structures.

##### effect of compression on feature engineering

When the graph is uncompressed, we can observe many connections and chains of pointers in the graph.

![uncompressed graph](./graphs/keep_img/test_graph_from_302-1644391327_uncompressed_no_vn-sfdp.png)

In this situation, generating features using several ancestors fo a given ValueNode or KeyNode (which are always the leaves of the graph), is meaningful since it can capture information about the location of the leaf in the graph. (It caputes information about every ancestor location starting at the leaf. The further we are from the leaf, the less relevant the information is, so there is a need for a theshold (depth) to stop the feature generation from ancestors).

However, when the graph is compressed, all the graphs only have at most 1 pointer in between an leaf and a DTN. The graph structure disappear and is compressed inside the weight values in pointer edges. This makes any depth superior to 1 (one pointer edge) irrelevant. (In case we would like to go up to the DTN, this would makes some samples having more features than others (for the leaves that are further than shallow ones). So for consistency, we must stay at a depth of 1).

![compressed graph](./graphs/keep_img/test_graph_from_302-1644391327_compressed_no_vn-sfdp.png)

> For simplicity, we thus decide to overwrite any depth value to 1 if the graph compression is active.

### `REMOVE_TRIVIAL_ZERO_SAMPLES`

##### simplifying the workflow of data by removing the lines of trivial full-of-zeros samples

When generating the samples for value nodes, it appears that most lines are as the following examples:

```shell
592,43,33,40,1,0,0,0,0,0,0,0,0
592,45,33,40,1,0,0,0,0,0,0,0,0
...
32784,2435,0,4097,1,0,0,0,0,0,0,0,0
32784,2436,0,4097,1,0,0,0,0,0,0,0,0
...
32784,761,0,4097,1,0,0,0,0,0,0,0,0
32784,762,0,4097,1,0,0,0,0,0,0,0,0
...
```

In order to limit the imbalancing of the dataset, and since these lines are just full of zeros, it is not very interesting to make our models take them as parameters for the training. So we have added a parameter to remove them directly at the moment we create them.

These lines probably represents part of strings or text data or even arrays of values that do not need to be accessed directly.
