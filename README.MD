# Obsidian

Your identity, your server, your rules.

Obsidian is an Identity Provider built from the ground up in Rust — no JVM, no Node, no framework doing the heavy lifting for you. Just a hand-rolled HTTP server on raw TCP, compiled down to one binary you can drop anywhere and run.

Most self-hosted IdPs make you pay for "free" software with your RAM and your patience. Keycloak wants a JVM and a coffee break to boot. Obsidian starts in milliseconds and fits comfortably on a Raspberry Pi. If you're going to own your identity infrastructure, it shouldn't feel like a punishment.

## Why this exists

Identity shouldn't live behind a paywall or inside a black box you can't read. Obsidian is open source, will stay open source, and is built so you can actually understand what it's doing — the whole HTTP stack is written from scratch, no hidden layers, no vendor lock-in, no "enterprise tier" holding features hostage.

If you self-host, this is for you.

## What you get

- **One binary.** Copy it, run it, done. No dependency hell.
- **Starts instantly.** No JVM warmup, no bloated init sequence.
- **Runs light.** Comfortable on a Pi, effortless on anything bigger.
- **Fully readable.** Every layer of the server is hand-written - nothing you can't audit yourself.

## Where it's at

Still early. This is a real project being built in the open, not a polished 1.0 yet.

Done:
- TCP server handling concurrent connections
- Full HTTP request parsing - request line, headers, body
- Query string parsing with percent-decoding
- Response handling
- Security hardening: timeouts, size limits on headers/body/URI, strict header validation

Coming:
- OIDC — authorization code flow, token endpoint, discovery
- WebAuthn / passkeys
- An admin UI worth looking at
- Persistent storage

## Running it

```bash
git clone git@github.com:<your-username>/obsidian.git
cd obsidian
cargo run
```

Listens on `127.0.0.1:<port>` by default.

## Where it's headed

Short term: finish the OIDC flow and put Obsidian to work as the real IdP for a service in my own homelab — not a demo, an actual replacement for Keycloak.

After that: passkeys as a first-class citizen from day one, an admin panel that doesn't look like it's from 2012, and hard numbers on memory and startup time against Keycloak and Authentik.

## License

TBD — will be open source, always.
