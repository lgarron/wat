use indicatif::ProgressBar;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    io::{stdout, Write},
    process::{Command, Stdio},
};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
struct ExitNodeStatus {
    #[serde(rename = "ID")]
    id: String,
    online: bool,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
struct Peer {
    #[serde(rename = "ID")]
    id: String,
    host_name: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
struct TailscaleStatusJSON {
    exit_node_status: Option<ExitNodeStatus>,
    peer: HashMap<String, Peer>,
}

fn tailscale_message() -> String {
    let Ok(mut child) = Command::new("/Applications/Tailscale.app/Contents/MacOS/Tailscale")
        .args(["status", "--json"])
        // .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    else {
        return "❌ <<< Could not spawn child process. >>>".to_owned();
    };

    let Ok(child_waited) = child.wait() else {
        return "❌  <<< Failed to wait on child process. >>>".to_owned();
    };

    if child_waited.code() != Some(0) {
        return "🚫 down".to_owned();
    }

    let Some(stdout) = child.stdout else {
        return "<<< Could not get stdout from child process. >>>".to_owned();
    };

    let Ok(status): Result<TailscaleStatusJSON, serde_json::Error> =
        serde_json::from_reader(stdout)
    else {
        return "<<< ❌ Could not parse status JSON. >>>".to_owned();
    };

    if let Some(exit_node_status) = &status.exit_node_status {
        let exit_node_id = &exit_node_status.id;
        for peer in status.peer.values() {
            if peer.id == *exit_node_id {
                return if exit_node_status.online {
                    format!("🆙 up (➡️ using online exit node: {})", peer.host_name)
                } else {
                    format!("🆙 up (‼️ using OFFLINE exit node: {})", peer.host_name)
                };
            }
        }
        "🆙 up (‼️ unidentifiable exit node)".to_string()
    } else {
        "🆙 up (no exit node)".to_string()
    }
}

pub(crate) fn tailscale(progress_bar: ProgressBar) {
    progress_bar.set_message(tailscale_message());
    stdout().flush().unwrap();
}
