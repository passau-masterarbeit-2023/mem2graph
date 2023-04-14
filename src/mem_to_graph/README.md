# Mem to Graph



## program params

`COMPRESS_POINTER_CHAINS`: bool. Activate or not the pointer compression. This actually means that chains of pointers (i.e. pointers that points to pointers) are compressed in a single edge, where the weight depends on the number of compressed PTR2PTR edges.

> When the compression is active, we no more have links between 2 pointers, and thus, we lose the connetions between different data structures.
