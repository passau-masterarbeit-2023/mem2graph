# Mem to Graph

## program params

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
