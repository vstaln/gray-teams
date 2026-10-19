//! teams — Microsoft Teams channel via a Power Automate "post to a
//! channel" workflow webhook (outbound). Config:
//! ~/.gray/teams/config.json {"webhook"} or TEAMS_WEBHOOK.
//! Inbound needs a Bot Framework registration — README notes the gap.

use serde_json::{json, Value};
use std::io::{BufRead, Write};

const NAME: &str = "teams";

fn manifest() -> Value {
    json!({
        "name": NAME, "display_name": "Microsoft Teams",
        "version": env!("CARGO_PKG_VERSION"), "protocol": "1.1",
        "tools": [
            {"name": "teams_send",
             "description": "Post text (or an Adaptive Card body) to the configured Teams channel webhook.",
             "parameters": {"type": "object", "properties": {
                "text": {"type": "string"}}, "required": ["text"]}},
        ],
        "commands": ["/teams"], "subcommands": ["teams"],
    })
}

fn config_path() -> std::path::PathBuf {
    let home = std::env::var_os("GRAY_HOME")
        .map(std::path::PathBuf::from)
        .or_else(|| std::env::home_dir().map(|h| h.join(".gray")))
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    home.join("teams").join("config.json")
}
fn webhook() -> Option<String> {
    std::env::var("TEAMS_WEBHOOK")
        .ok()
        .filter(|s| !s.is_empty())
        .or_else(|| {
            std::fs::read_to_string(config_path())
                .ok()
                .and_then(|s| serde_json::from_str::<Value>(&s).ok())
                .and_then(|c| c["webhook"].as_str().map(str::to_string))
        })
}

fn send(a: &Value) -> Result<String, String> {
    let url = webhook().ok_or("no webhook — `teams setup --webhook https://…logic.azure.com/…`")?;
    // Power Automate workflow webhooks accept an Adaptive Card envelope.
    let body = json!({
        "type": "message",
        "attachments": [{"contentType": "application/vnd.microsoft.card.adaptive",
            "content": {"type": "AdaptiveCard", "$schema":
                "http://adaptivecards.io/schemas/adaptive-card.json",
                "version": "1.4", "body": [{"type": "TextBlock",
                    "text": a["text"].as_str().ok_or("text required")?,
                    "wrap": true}]}}]
    });
    match ureq::post(&url)
        .set("Content-Type", "application/json")
        .timeout(std::time::Duration::from_secs(15))
        .send_json(body)
    {
        Ok(_) => Ok("posted".into()),
        Err(ureq::Error::Status(c, r)) => Err(format!(
            "HTTP {c}: {}",
            r.into_string()
                .unwrap_or_default()
                .chars()
                .take(250)
                .collect::<String>()
        )),
        Err(e) => Err(e.to_string()),
    }
}

fn cli(argv: &[String]) -> Result<String, String> {
    match argv.first().map(String::as_str) {
        Some("setup") => {
            let mut it = argv[1..].iter();
            while let Some(f) = it.next() {
                if f == "--webhook" {
                    let w = it.next().ok_or("--webhook needs URL")?;
                    std::fs::create_dir_all(config_path().parent().unwrap())
                        .map_err(|e| e.to_string())?;
                    std::fs::write(
                        config_path(),
                        serde_json::to_string_pretty(&json!({"webhook": w})).unwrap(),
                    )
                    .map_err(|e| e.to_string())?;
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;
                        let _ = std::fs::set_permissions(
                            config_path(),
                            std::fs::Permissions::from_mode(0o600),
                        );
                    }
                    return Ok("saved".into());
                }
                return Err(format!("unknown flag {f}"));
            }
            Err("setup needs --webhook".into())
        }
        Some("status") | None => Ok(format!(
            "Teams\n  webhook: {}",
            if webhook().is_some() {
                "set"
            } else {
                "MISSING"
            }
        )),
        Some("-h" | "--help" | "help") => Ok("usage: teams setup --webhook URL | status".into()),
        Some(o) => Err(format!("unknown verb {o}")),
    }
}

fn sidecar() -> std::io::Result<()> {
    let stdin = std::io::stdin();
    let mut out = std::io::stdout().lock();
    for line in stdin.lock().lines() {
        let line = line?;
        let Ok(v) = serde_json::from_str::<Value>(&line) else {
            continue;
        };
        let method = v["method"].as_str().unwrap_or("");
        match v.get("id").cloned() {
            None if method == "plugin/shutdown" => return Ok(()),
            None => {}
            Some(id) => {
                let reply = if method == "plugin/manifest" {
                    json!({"id": id, "result": manifest()})
                } else if method == "tool/call" {
                    let r = match send(&v["params"]["args"]) {
                        Ok(c) => json!({"id": id, "result": {"content": c}}),
                        Err(e) => json!({"id": id, "result": {"content": e, "is_error": true}}),
                    };
                    r
                } else if method == "command/run" {
                    let argv: Vec<String> = v["params"]["argv"]
                        .as_array()
                        .map(|a| {
                            a.iter()
                                .filter_map(Value::as_str)
                                .map(str::to_string)
                                .collect()
                        })
                        .unwrap_or_default();
                    let text = cli(&argv).unwrap_or_else(|e| e);
                    json!({"id": id, "result": {"text": text}})
                } else {
                    json!({"id": id, "error": {"code": -32601,
                        "message": format!("method not found: {method}")}})
                };
                writeln!(out, "{reply}")?;
                out.flush()?;
            }
        }
    }
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args: Vec<String> = match args.first().map(String::as_str) {
        Some("teams") => args[1..].to_vec(),
        _ => args,
    };
    match args.first().map(String::as_str) {
        Some("manifest") => println!("{}", manifest()),
        Some("sidecar") => {
            let _ = sidecar();
        }
        _ => {
            if args.is_empty() {
                if std::io::IsTerminal::is_terminal(&std::io::stdin()) {
                    println!("teams — outbound webhook channel. `teams setup --webhook URL`");
                    return;
                }
                let _ = sidecar();
                return;
            }
            match cli(&args) {
                Ok(t) => println!("{t}"),
                Err(e) => {
                    eprintln!("{e}");
                    std::process::exit(1);
                }
            }
        }
    }
}
