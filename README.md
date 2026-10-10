<div align="center">
  <img alt="gray-teams" src="assets/icon.svg" width="120" height="120" />
  <h1>gray-teams</h1>
  <p><strong>Microsoft Teams channel posts via a Power Automate workflow webhook.</strong></p>
  <p>
    <a href="https://gray.alignment.id">Website</a> ·
    <a href="https://github.com/vstaln/gray-teams">Source</a> ·
    <a href="https://github.com/vstaln/gray">gray</a>
  </p>
  <p>
    <a href="LICENSE"><img alt="License: MIT" src="https://img.shields.io/badge/license-MIT-1c1c20?style=flat-square&labelColor=0a0a0b" /></a>
    <a href="https://www.rust-lang.org"><img alt="Built with Rust" src="https://img.shields.io/badge/built%20with-rust-1c1c20?style=flat-square&labelColor=0a0a0b&logo=rust&logoColor=d4a373" /></a>
    <img alt="gray plugin" src="https://img.shields.io/badge/gray-plugin-1c1c20?style=flat-square&labelColor=0a0a0b&color=7aa2f7" />
  </p>
</div>

<br/>
```bash
gray plugin install teams
```

`gray-teams` posts Adaptive Card messages into a Teams channel through a
**Power Automate "post to a channel" workflow webhook** — the current
supported shape for webhook posting (the legacy Office 365 connector was
deprecated).

## Setup

```sh
teams setup --webhook 'https://…logic.azure.com/workflows/…'
teams status
```

Config: `~/.gray/teams/config.json` (`{"webhook"}`) or env
`TEAMS_WEBHOOK`. Create the webhook with the Power Automate template
"Post to a channel when a webhook request is received" — its trigger URL
is what you paste here.

## Tool

| Tool | What it does |
|---|---|
| `teams_send` | Post text as an Adaptive Card — `text`. |

## Inbound?

Not this plugin. True inbound (the agent being @-mentioned, reading
channel traffic) needs a Bot Framework registration + messaging endpoint —
a bigger integration this one deliberately doesn't pretend to be.

## Build from source

```sh
cargo build --release
gray plugin check ./target/release/teams
```

## License

MIT
