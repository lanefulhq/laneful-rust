# Examples

## Prerequisites

Set environment variables:

```bash
export LANEFUL_ENDPOINT="https://your-laneful-endpoint.com"
export LANEFUL_API_KEY="your-api-key"
```

## Sync Example

```bash
cargo run --example sync -- --from sender@example.com --to recipient@example.com
```

## Async Example

```bash
cargo run --example async --features async -- --from sender@example.com --to recipient@example.com
```

## Webhook Server

A server that verifies and logs incoming webhook requests:

```bash
# Set webhook secret (defaults to "test-secret" if not set)
export LANEFUL_WEBHOOK_SECRET="your-webhook-secret"

cargo run --example webhook_server

# Or with custom port:
PORT=8080 cargo run --example webhook_server
```

### External Access via Tunnel

To test webhooks from external services, create a tunnel using [pinggy.io](https://pinggy.io):

```bash
# Start the webhook server (default port 9090)
cargo run --example webhook_server

# In another terminal, create tunnel to expose localhost:9090
ssh -p 443 -R0:localhost:3344 qr@free.pinggy.io
```

Pinggy will output a public URL (e.g., `https://xyz.free.pinggy.link`) that forwards to your local webhook server.
