# Logs

Logs powered by the [tracing](https://crates.io/crates/tracing) and [tracing-subscriber](https://crates.io/crates/tracing-subscriber) crate.

Logs are configured through the `RUST_LOG` environment variable. [Example Syntax](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html#example-syntax)

## Examples

### Default

```
RUST_LOG=info
```

![logs](./logs.svg)

### Per Module

```
RUST_LOG=trace,hyper=debug
```

![logs per module](./logs_per_module.svg)
