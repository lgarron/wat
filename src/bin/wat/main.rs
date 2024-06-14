// TODO: this code is pretty "wat" in itself. Once I understand the full structure of what I need, I should refactor it.

use std::{
    io::{stdout, Write},
    process::{Command, Stdio},
    str::from_utf8,
    sync::Arc,
    thread::{self, JoinHandle},
    time::Duration,
};

mod options;

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use linereader::LineReader;
use options::get_options;

fn main() {
    let wat_args = get_options();

    let multi_progress = Arc::new(MultiProgress::new());

    let create_progress_bar = move |prefix: &str| -> ProgressBar {
        let progress_bar = ProgressBar::new(200).with_finish(indicatif::ProgressFinish::AndLeave);
        let progress_bar = multi_progress.insert_from_back(0, progress_bar);
        progress_bar.set_style(
            ProgressStyle::with_template("{elapsed:>06} | {prefix} | {wide_msg}")
                .expect("Could not construct progress bar template."),
        );
        progress_bar.set_prefix(prefix.to_owned());
        progress_bar.tick();
        progress_bar.enable_steady_tick(Duration::from_millis(100));
        progress_bar
    };

    let mut handles: Vec<JoinHandle<()>> = vec![];

    macro_rules! spawn {
        ($prefix:expr, $fn:expr) => {
            handles.push({
                let progress_bar = create_progress_bar($prefix);
                thread::spawn(move || {
                    $fn(progress_bar);
                })
            });
        };
    }

    macro_rules! spawn_1arg {
        ($prefix:expr, $fn:expr, $arg:expr) => {
            handles.push({
                let progress_bar = create_progress_bar($prefix);
                thread::spawn(move || {
                    $fn(progress_bar, $arg);
                })
            });
        };
    }

    if wat_args.include_system() {
        spawn!("Charging", charging);
        spawn!("🔥 thermal pressure", thermal_pressure);
        spawn!("💾 disk space free", disk_space_free);
        spawn!("🔈 audio output", audio_output);
        spawn_1arg!("IPv4 address (WiFi)", ipv4_address, &["en0"]);
        spawn_1arg!(
            "IPv4 address (other)",
            ipv4_address,
            &["en17", "en23", "bridge0"]
        );
        spawn!("tailscale", tailscale);
    }
    if wat_args.include_misc() {
        spawn_1arg!(
            "Pythagoras.tlb locked?",
            ssh_is_macos_screen_locked,
            "Pythagoras.tlb"
        );
        spawn_1arg!(
            "Pythagoras-ts.wyvern-climb.ts.net locked?",
            ssh_is_macos_screen_locked,
            "Pythagoras-ts.wyvern-climb.ts.net"
        );
    }
    if wat_args.include_ping() {
        spawn_1arg!("ping 1.1.1.1", ping, "1.1.1.1");
        spawn_1arg!("ping 8.8.8.8", ping, "8.8.8.8");
        spawn_1arg!("ping mensura.cdn-apple.com", ping, "mensura.cdn-apple.com");
        spawn_1arg!("ping Pythagoras.tlb", ping, "Pythagoras.tlb");
        spawn_1arg!(
            "ping Pythagoras-ts.wyvern-climb.ts.net",
            ping,
            "Pythagoras-ts.wyvern-climb.ts.net"
        );
    }

    if wat_args.include_sshping() {
        spawn_1arg!("sshping Pythagoras.tlb", sshping, "Pythagoras.tlb");
        spawn_1arg!(
            "sshping Pythagoras-ts.wyvern-climb.ts.net",
            sshping,
            "Pythagoras-ts.wyvern-climb.ts.net"
        );
    }

    for handle in handles.drain(1..) {
        handle.join().unwrap();
    }

    if wat_args.include_network_quality() {
        spawn!("networkQuality", network_quality);
    }
    if wat_args.include_iperf3() {
        spawn_1arg!("Pythagoras.tlb iperf3", iperf3, "Pythagoras.tlb");
    }
    if wat_args.include_iperf3_tailscale() {
        spawn_1arg!(
            "Pythagoras-ts.wyvern-climb.ts.net iperf3",
            iperf3,
            "Pythagoras-ts.wyvern-climb.ts.net"
        );
    }
    for handle in handles {
        handle.join().unwrap();
    }
}

fn network_quality(progress_bar: ProgressBar) {
    // progress_bar.set_prefix("starting");
    let child = Command::new("faketty")
        .args(["networkQuality"])
        // .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();

    let mut line_reader = LineReader::with_delimiter(b'\r', child.unwrap().stdout.unwrap());
    line_reader
        .for_each(|line| {
            let line = from_utf8(line).unwrap().to_owned();
            if !line.starts_with("Downlink") {
                return Ok(true);
            }
            progress_bar.set_message(line);
            stdout().flush().unwrap();
            Ok(true)
        })
        .unwrap();
}

fn iperf3(progress_bar: ProgressBar, host: &str) {
    let server_ssh_process: Result<std::process::Child, std::io::Error> = Command::new("ssh")
        .args([host, "iperf3 --server"])
        // .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();

    // progress_bar.set_prefix("starting");
    let child = Command::new("faketty")
        .args(["iperf3", "--client", host])
        // .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();

    let mut line_reader = LineReader::with_delimiter(b'\r', child.unwrap().stdout.unwrap());
    line_reader
        .for_each(|line| {
            let line = from_utf8(line).unwrap().trim().to_owned();
            if line.is_empty() {
                return Ok(true);
            }
            if line == "iperf Done." {
                return Ok(true);
            }
            progress_bar.set_message(line);
            stdout().flush().unwrap();
            Ok(true)
        })
        .unwrap();

    server_ssh_process.unwrap().kill().unwrap();
}

fn ping(progress_bar: ProgressBar, host: &str) {
    // progress_bar.set_prefix("starting");
    let child = Command::new("faketty")
        .args(["ping", "-c", "8", host])
        // .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();

    let mut line_reader = LineReader::with_delimiter(b'\r', child.unwrap().stdout.unwrap());
    line_reader
        .for_each(|line| {
            let line = from_utf8(line).unwrap().trim().to_owned();
            if line.is_empty() {
                return Ok(true);
            }
            progress_bar.set_message(line);
            stdout().flush().unwrap();
            Ok(true)
        })
        .unwrap();
}

fn ssh_is_macos_screen_locked(progress_bar: ProgressBar, host: &str) {
    let child = Command::new("ssh")
        .args([host, "\"is-macos-screen-locked\""])
        // .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();

    let output = child.unwrap().wait_with_output().unwrap();

    let output = String::from_utf8(output.stdout).unwrap();
    let output = if output.contains("unlocked") {
        "🚫 unlocked"
    } else if output.contains("locked") {
        "🔒 locked"
    } else {
        "—"
    };
    progress_bar.set_message(output);
}

fn audio_output(progress_bar: ProgressBar) {
    let child = Command::new("SwitchAudioSource")
        .args(["-c"])
        // .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();

    let output = child.unwrap().wait_with_output().unwrap();

    let output = String::from_utf8(output.stdout).unwrap();
    let output = output.trim().to_owned();
    // println!("---{}===", output);
    progress_bar.set_message(output);
}

fn sshping(progress_bar: ProgressBar, host: &str) {
    let child = Command::new("/Users/lgarron/Code/git/github.com/spook/sshping/bin/sshping")
        .args(["-H", "--time", "10", host])
        // .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();

    let mut line_reader = LineReader::with_delimiter(b'\r', child.unwrap().stdout.unwrap());
    line_reader
        .for_each(|line| {
            let line = from_utf8(line).unwrap().to_owned();
            progress_bar.set_message(line);
            stdout().flush().unwrap();
            Ok(true)
        })
        .unwrap();
}

fn tailscale(progress_bar: ProgressBar) {
    let child = Command::new("/Applications/Tailscale.app/Contents/MacOS/Tailscale")
        .args(["status"])
        // .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .status();

    progress_bar.set_message(if child.unwrap().code() == Some(0) {
        "↔️ up"
    } else {
        "🚫 down"
    });
    stdout().flush().unwrap();
}

fn ipv4_address(progress_bar: ProgressBar, interfaces: &[&str]) {
    for interface in interfaces {
        let child = Command::new("ipconfig")
            .args(["getifaddr", interface])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn();

        let output = child.unwrap().wait_with_output().unwrap();

        let output = String::from_utf8(output.stdout).unwrap();
        let output = output.trim().to_owned();
        if !output.is_empty() {
            progress_bar.set_message(format!("{} ({})", output, interface));
            return;
        }
    }
    progress_bar.set_message("—");
}

fn charging(progress_bar: ProgressBar) {
    let child = Command::new(
        "/Users/lgarron/Code/git/github.com/lgarron/scripts/system/macos-charging-watts.fish",
    )
    // .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn();

    let mut line_reader = LineReader::with_delimiter(b'\r', child.unwrap().stdout.unwrap());
    line_reader
        .for_each(|line| {
            let line = from_utf8(line).unwrap().to_owned();
            progress_bar.set_message(line);
            stdout().flush().unwrap();
            Ok(true)
        })
        .unwrap();
}

fn disk_space_free(progress_bar: ProgressBar) {
    let child = Command::new("df")
        .args(["-h"])
        // .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();

    let mut is_first_line = true;
    let mut line_reader = LineReader::new(child.unwrap().stdout.unwrap());
    line_reader
        .for_each(|line| {
            if is_first_line {
                is_first_line = false;
                return Ok(true);
            }
            let line = from_utf8(line).unwrap().to_owned();
            // TODO: Why can't this be a `Vec<&str>`?
            let columns: Vec<String> = line.split_whitespace().map(|s| s.to_owned()).collect();
            progress_bar.set_message(columns[3].clone());
            stdout().flush().unwrap();
            Ok(false)
        })
        .unwrap();
}

fn thermal_pressure(progress_bar: ProgressBar) {
    let child = Command::new("thermal-pressure")
        // .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();

    let mut line_reader = LineReader::with_delimiter(b'\r', child.unwrap().stdout.unwrap());
    line_reader
        .for_each(|line| {
            let mut line = from_utf8(line).unwrap().to_owned();
            if let Some(prefix_stripped_line) = line.strip_prefix("Current pressure level: ") {
                line = prefix_stripped_line.to_owned()
            }
            progress_bar.set_message(line);
            stdout().flush().unwrap();
            Ok(true)
        })
        .unwrap();
}
