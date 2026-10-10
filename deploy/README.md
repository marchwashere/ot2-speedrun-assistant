# Debian deployment (personal, authenticated service)

This uses the existing Rust HTTP executable, unchanged simulation libraries and
embedded HTML/CSS/JS. Caddy runs on the same host. There is no Node.js runtime,
game installation, tracer, or extracted game directory required to build/run.
The import/audit tools are development tools and do require the local source assets;
they are NOT run during deployment. This server processes one request at a time.
Keep Caddy authentication enabled; it is not yet a public multi-user job server.

## 1. Prerequisites and authentication

As your normal SSH account (not root), install build dependencies:

```sh
sudo apt update
sudo apt install -y git gh build-essential curl ca-certificates pkg-config
```

Install current stable Rust using https://rustup.rs/ if it is not already installed.
Then `source "$HOME/.cargo/env"` (or reconnect SSH) and check `rustc --version`.
Authenticate with `gh auth login --hostname github.com`, choosing the marchwashere
account and HTTPS, then `gh auth setup-git`. Both repositories are private.
Never put tokens in source files, clone URLs, or the Caddyfile.

## 2. Clone and build the pinned pair

```sh
mkdir -p ~/ot2-src
cd ~/ot2-src
gh repo clone marchwashere/ot2-speedrun-assistant OT2SpeedrunAssistant
gh repo clone marchwashere/ot2-battle-engine OT2BattleEngineRust
cd OT2BattleEngineRust
git checkout --detach "$(cat ../OT2SpeedrunAssistant/deploy/battle-core-revision)"
cd ../OT2SpeedrunAssistant
cargo build --locked --release --bin ot2-gui
cargo test --locked --release --bin ot2-gui hosted_origin_and_shutdown_policy
```

Keep the sibling directory names exactly as above: Cargo's path dependency uses
`../OT2BattleEngineRust`. The revision file pins the pairing without preventing
normal side-by-side local development. Do not blindly pull the core's latest main.

## 3. Install the service

From the assistant directory:

```sh
id ot2 >/dev/null 2>&1 || sudo useradd --system --user-group --home-dir /nonexistent --shell /usr/sbin/nologin ot2
sudo install -d -m 0755 /opt/ot2
sudo install -m 0755 target/release/ot2-gui /opt/ot2/ot2-gui
sudo install -m 0644 deploy/ot2.service /etc/systemd/system/ot2.service
sudo install -m 0644 deploy/ot2.env.example /etc/ot2.env
sudoedit /etc/ot2.env
```

Set `OT2_PUBLIC_ORIGIN=https://YOUR.DOMAIN` with no trailing slash. This enables
public Host/Origin validation, hides the local Quit button, and disables shutdown.
No proxy header rewriting is necessary. Leave the backend on127.0.0.1:54345.

```sh
sudo systemctl daemon-reload
sudo systemctl enable --now ot2
sudo systemctl status ot2 --no-pager
curl -f -H 'Host: YOUR.DOMAIN' http://127.0.0.1:54345/api/health
```

The explicit16MiB simulation-thread stack is already in the executable.

## 4. Caddy

Add deploy/Caddyfile.example's site block to your existing Caddy configuration.
Replace the domain and generate a password hash using `caddy hash-password`
(interactive). Paste the hash, not the plaintext password, for user runner.
Point DNS at this host and allow inbound80/443; do NOT expose54345.

```sh
sudo caddy validate --config /etc/caddy/Caddyfile
sudo systemctl reload caddy
curl -f -u runner https://YOUR.DOMAIN/api/health
```

The curl command prompts for the password. Visit the HTTPS URL and run a known
seed before using the hosted instance for a run. Caddy handles HTTPS; frontend
requests use same-origin `/api/...` paths.

## Updates and rollback

Pull the assistant with `git pull --ff-only`, then in the core run `git fetch origin`
and check out the new revision specified by deploy/battle-core-revision. Rebuild
with --locked and test before replacing the service. For installation:

```sh
sudo install -m 0755 target/release/ot2-gui /opt/ot2/ot2-gui.next
sudo systemctl stop ot2
sudo cp -p /opt/ot2/ot2-gui /opt/ot2/ot2-gui.previous
sudo mv /opt/ot2/ot2-gui.next /opt/ot2/ot2-gui
sudo systemctl start ot2
```

Check health and a known route. To roll back, stop the service, restore
`ot2-gui.previous` to`ot2-gui`, and start again.
Logs: `journalctl -u ot2 -n100 --no-pager`.

## Validation scope

Hosted Host/Origin/shutdown tests pass on the development Windows build. A Linux
build and live Caddy/TLS verification must run on your Debian host; they have not
been claimed as completed. Broader battle/route validation is recorded in project
docs. The existing single-worker server intentionally serializes simulations;
concurrent public users need a separate bounded-worker/job-server upgrade.
