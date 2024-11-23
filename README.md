# atr
Another trace route that allows tracing the route taken by packets as they travel from source to destination

# Usage
After `cargo build`, run `atr`. e.g.

`target/debug/atr -t stackpointer.dev:443`

`atr` expects two arguments
- the protocol, e.g. `t` for TCP
- the host name with port
