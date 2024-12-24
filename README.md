# Implementation of Berkley DB

Run the following command:

```bash
./gendata.py | db_load -T -t btree testdata.bdb
```

run the following command:
```bash
cd bdb-rs && cargo run -- ../testdata.bdb
```

reference to the following link for more information: https://transactional.blog/building-berkeleydb/


## Overview
## Why BerkeleyDB?
BerkeleyDB is ubiquitous. It’s installed on every platform, or easily installable, and there’s bindings available to it in nearly every language.

BerkeleyDB is simple. It’s not a highly optimized B-Tree implementation. 
It isn’t tuned or written with any one data model in mind. The B-Tree is a 
plain key-value store.


BerkeleyDB is realistic. It’s been used in real-world applications. 
Features were added to it according to the needs of real software. 
It influenced the design and of other embedded databases. What one learns 
from BerkeleyDB will be applicable to other, more complicated B-Trees.

### Metapage 

This is the metapage

![metapage](./metapage.svg)

The gap start on byte 460.
The entire structure is 512 bytes in size. 
The rest of the 4096 byte page is unspecified and unused. 

12-15 byte range: Magic Number
It should always be 0x53162. This magic number is also unique to the Btree data type, so requiring this magic number makes sure that the file is not one of the alternative access methods supported by BerkeleyDB (hash, record, or queue).

28-31 byte range: Free list page number
This will be used once mutations are implemented on the btree.

32-35 byte range: Last page number
The page number of the last page in the database. This can be used instead of relying on size_of_database_file / 4KB - 1.

88-91: Root
The page number of the root of the btree. Reads of the btree should start from this page.


### BTree page

![btree](./btree.svg)

The rest of the page is used to hold the page entries, which is the subject of our next step. For now, we only focus on the page headers. No bolding of fields this time, as all of them will be used.

00-07: Log sequence number
The sequence number of this page, which will become important once updates and WAL support is implemented.

08-11: Current page number
The page number of this page.

12-15: Previous page number
The leaf page containing lexicographically lower data. 0 if none, or if internal node.

16-19: Next page number
The leaf page containing lexicographically higher data. 0 if none, or if internal node.

20-21: Number of items on the page
The number of entries contained on this page.

22-23: High free byte page offset
The space between this byte offset and the header is empty.

24: Btree tree level
A leaf node is 1, and it counts up, so any value greater than 1 also means an internal page.

25: Page type
The DB meta page is type 9. An internal node is type 3. A leaf node is type 5. All other values are invalid.


### Page Entries



![page](./page.svg)

# References

https://transactional.blog/building-berkeleydb/
