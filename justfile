malestrom_bin := "./maelstrom/maelstrom"

default:
    @just --choose

# serve the maelstrom server
serve:
    {{ malestrom_bin }} serve

build-echo:
    cargo build --release --bin echo

test-echo: (build-echo)
    {{ malestrom_bin }} test -w echo --bin ./target/release/echo --node-count 1 --time-limit 10

build-unique-ids:
    cargo build --release --bin unique-ids

test-unique-ids: (build-unique-ids)
    {{ malestrom_bin }} test -w unique-ids --bin ./target/release/unique-ids --time-limit 30 --rate 1000 --node-count 3 --availability total --nemesis partition

build-broadcast:
    cargo build --release --bin broadcast

test-broadcast-a: (build-broadcast)
    {{ malestrom_bin }} test -w broadcast --bin ./target/release/broadcast --node-count 1 --time-limit 20 --rate 10

test-broadcast-b: (build-broadcast)
    {{ malestrom_bin }} test -w broadcast --bin ./target/release/broadcast --node-count 5 --time-limit 20 --rate 10

test-broadcast-c: (build-broadcast)
    {{ malestrom_bin }} test -w broadcast --bin ./target/release/broadcast --node-count 5 --time-limit 20 --rate 10 --nemesis partition
