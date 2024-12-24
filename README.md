# Implementation of Berkley DB

Run the following command:

```bash
./gendata.py | db_load -T -t btree testdata.bdb
```

run the following command:
```bash
cd bdb-rs && cargo run -- ../testdata.bdb
```


# References

https://transactional.blog/building-berkeleydb/
