# dns-block

A _work-in-progress_ DNS blocker.

Start via `cargo run -p dns-block` and query DNS records via `dig google.de @127.0.0.1 -p 53000`.
For now only `google.de` is blocked by immediately returning a `NXDOMAIN` response (opcode 3).
