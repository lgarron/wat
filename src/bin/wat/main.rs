// TODO: this code is pretty "wat" in itself. Once I understand the full structure of what I need, I should refactor it.

use colored::{ColoredString, Colorize, CustomColor};
use std::{
    io::{stdout, Write},
    process::{Command, Stdio},
    str::from_utf8,
    sync::Arc,
    thread::{self, JoinHandle},
    time::Duration,
};

mod args;
mod tailscale;

use args::get_options;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use linereader::LineReader;
use regex::Regex;

use shadow_rs::shadow;

use crate::tailscale::tailscale;

shadow!(build);

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
            "Pythagoras.lan locked?",
            ssh_is_macos_screen_locked,
            "Pythagoras.lan"
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
        spawn_1arg!("ping Pythagoras.lan", ping, "Pythagoras.lan");
        spawn_1arg!(
            "ping Pythagoras-ts.wyvern-climb.ts.net",
            ping,
            "Pythagoras-ts.wyvern-climb.ts.net"
        );
    }

    if wat_args.include_sshping() {
        spawn_1arg!("sshping Pythagoras.lan", sshping, "Pythagoras.lan");
        spawn_1arg!(
            "sshping Pythagoras-ts.wyvern-climb.ts.net",
            sshping,
            "Pythagoras-ts.wyvern-climb.ts.net"
        );
    }

    if wat_args.speedtest_separately {
        for handle in handles.drain(0..) {
            handle.join().unwrap();
        }
    }

    if wat_args.include_speedtest() {
        spawn!("speedtest", speedtest);
    }
    if wat_args.include_iperf3() {
        spawn_1arg!("Pythagoras.lan iperf3", iperf3, "Pythagoras.lan");
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

const DOWNLOAD: &str = "Download:";

fn speedtest(progress_bar: ProgressBar) {
    // progress_bar.set_prefix("starting");
    let child = Command::new("faketty")
        .args(["speedtest"])
        // .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();

    let mut line_reader = LineReader::with_delimiter(b'\r', child.unwrap().stdout.unwrap());
    line_reader
        .for_each(|line| {
            let line = from_utf8(line).unwrap().trim().to_owned();
            if !line.starts_with("Download") {
                return Ok(true);
            }
            progress_bar.set_message(line[DOWNLOAD.len()..].trim().to_owned());
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
        .args(["-o", "ConnectTimeout=5", host, "\"is-macos-screen-locked\""])
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
    let child = Command::new("sshping")
        .args([
            "--table-style",
            "blank",
            "--echo-timeout",
            "5",
            "--ssh-timeout",
            "5",
            host,
        ])
        // .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();

    let mut average_time: Option<Duration> = None;
    let mut connect_time: Option<Duration> = None;
    let mut std_dev_time: Option<Duration> = None;

    let mut line_reader = LineReader::with_delimiter(b'\r', child.unwrap().stdout.unwrap());
    line_reader
        .for_each(|line| {
            let line = from_utf8(line).unwrap().to_owned();

            let re = Regex::new(r"Connect time *([0-9,]+)ns").unwrap();
            if let Some(captures) = re.captures(&line) {
                let (_, [nanoseconds_str]) = captures.extract();
                connect_time = Some(Duration::from_micros(
                    nanoseconds_str.replace(",", "").parse::<u64>().unwrap() / 1000,
                ));
            }

            let re = Regex::new(r"Average *([0-9,]+)ns").unwrap();
            if let Some(captures) = re.captures(&line) {
                let (_, [nanoseconds_str]) = captures.extract();
                average_time = Some(Duration::from_micros(
                    nanoseconds_str.replace(",", "").parse::<u64>().unwrap() / 1000,
                ));
            }

            let re = Regex::new(r"Std deviation *([0-9,]+)ns").unwrap();
            if let Some(captures) = re.captures(&line) {
                let (_, [nanoseconds_str]) = captures.extract();
                std_dev_time = Some(Duration::from_micros(
                    nanoseconds_str.replace(",", "").parse::<u64>().unwrap() / 1000,
                ));
            }

            let mut parts: Vec<String> = vec![];
            if let Some(average_time) = average_time {
                parts.push(format!("⌀ {:?}", average_time));
            }
            if let Some(std_dev_time) = std_dev_time {
                parts.push(format!("± 𝜎 {:?}", std_dev_time));
            }
            if let Some(connect_time) = connect_time {
                parts.push(format!("(↔ {:?})", connect_time));
            }
            progress_bar.set_message(parts.join(" "));
            stdout().flush().unwrap();

            Ok(true)
        })
        .unwrap();
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
        "/Users/lgarron/Code/git/github.com/lgarron/dotfiles/scripts/system/macos-charging-watts.ts",
    ).args(["--format", "number"])
    // .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn();

    let mut line_reader = LineReader::with_delimiter(b'\r', child.unwrap().stdout.unwrap());
    line_reader
        .for_each(|line| {
            let mut line = from_utf8(line).unwrap().to_owned().trim().to_owned();
            let Ok(num_watts) = line.parse::<u32>() else {
                return Ok(false);
            };

            let colored_msg = if num_watts <= 30 {
                format!("🚨 {} watts", num_watts).red()
            } else if num_watts < 60 {
                format!("⚠️ {} watts", num_watts).custom_color(CustomColor::new(255, 127, 0))
            } else if num_watts < 96 {
                format!("🔌 {} watts", num_watts).normal()
            } else if num_watts < 140 {
                format!("🔌 {} watts", num_watts).blue()
            } else if num_watts == 140 {
                format!("🔌 {} watts", num_watts).green()
            } else {
                format!("🔌 {} watts", num_watts).purple()
            };
            line = format!("{}", colored_msg);
            progress_bar.set_message(line);
            stdout().flush().unwrap();
            Ok(true)
        })
        .unwrap();
}

fn disk_space_free(progress_bar: ProgressBar) {
    let child = Command::new("df")
        .args(["-h", "--si"])
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

            let mut msg_string = columns[3].clone();
            // TODO: unclear if `Ki` is actually possible in the output?
            let re = Regex::new(r"([0-9]+)(K|M|G|T)").unwrap();
            if let Some(captures) = re.captures(&msg_string) {
                let (_, [num_gi_str, unit]) = captures.extract();
                let num_gi: usize = num_gi_str.parse().unwrap();
                let msg_string_spaced = format!("{} {}B", num_gi_str, unit);
                let colored_msg = if num_gi < 20 || unit == "M" || unit == "K" {
                    format!("🚨 {}", msg_string_spaced).red()
                } else if num_gi < 50 {
                    format!("⚠️ {}", msg_string_spaced).custom_color(CustomColor::new(255, 127, 0))
                } else if num_gi < 100 {
                    format!("🤔 {}", msg_string_spaced).yellow()
                } else if num_gi > 200 {
                    msg_string_spaced.green()
                } else {
                    msg_string_spaced.bold()
                };
                msg_string = format!("{}", colored_msg);
            }

            progress_bar.set_message(msg_string);
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
            progress_bar.set_message(
                auto_color_good_or_bad(line.trim(), "Nominal", "🔥 ")
                    .to_string()
                    .to_owned(),
            );
            stdout().flush().unwrap();
            Ok(true)
        })
        .unwrap();
}

fn auto_color_good_or_bad(s: &str, good_string: &str, bad_prefix: &str) -> ColoredString {
    if s == good_string {
        s.green()
    } else {
        format!("{}{}", bad_prefix, s).red()
    }
}
