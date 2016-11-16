# IRC library for Rust

This is a library for interacting with the IRC protocol via. Rust. There are
probably several of these libraries, but this one is my one.

Super-experimental, mostly a learning experience.

## Try it out

There's a runnable example in `examples/echo`. Try this (set `$nick` and
`$channel` to something else!):

```
cd examples/echo
nick=rustechobot
channel=#superhugs
cargo run chat.freenode.net 6697 ssl "$nick" "$channel"
```

## How's my driving?
This library is  primarily a way for me to learn Rust, so I'm especially
interested if anyone who reads this has tips on anything I'm doing wrong or
right here.
