# Rabx - Simple Persistent Key value store
## Description
Rabx, is a simple rust key value store, which stores data on disk in form of a write ahead log.

Currently the log compaction algorithm is very naive, but works as expected.

It offers both a cli for stateless access and library for stateful access.

The log data is stored in `BSON` format. The cli stores data by default at `./db.bson`

## INSTALL
```bash
curl --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/crazystylus/rabx/master/install | bash
```

## USAGE
```bash
$ kvs set foo bar
$ kvs get foo
bar

$ kvs rm foo
$ kvs get foo
Key not found
```
