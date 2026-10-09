# State

A minimal, hand-written state-machine coroutine, and a non-blocking HTTP client built on Metal I/O.

## How it works

- `src/future.rs` — a minimal `Future` trait and `PollState` enum.
- `src/main.rs` — a `Coroutine` state machine that chains two HTTP requests via polling.
- `src/http.rs` — an `HttpGetFuture` that connects, writes a request, and reads the response using non-blocking I/O.

The `main` loop polls the future and idles when it returns `NotReady`.

## Run

This program expects a HTTP server running at `127.0.0.1:8080`.

```sh
cargo run
```
