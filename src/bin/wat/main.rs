use std::{
    io::{stdout, Write},
    process::{Command, Stdio},
    str::from_utf8,
    sync::Arc,
    thread::{self, sleep},
    time::Duration,
};

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use linereader::LineReader;

fn main() {
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

    let handles = [
        {
            let progress_bar = create_progress_bar("🔈 audio output");
            thread::spawn(move || {
                audio_output(progress_bar);
            })
        },
        {
            let progress_bar = create_progress_bar("Pythagoras.tlb locked?");
            thread::spawn(move || {
                pythagoras_tlb_locked(progress_bar);
            })
        },
        {
            let progress_bar = create_progress_bar("Pythagoras-ts.wyvern-climb.ts.net locked?");
            thread::spawn(move || {
                pythagoras_ts_tailscale_locked(progress_bar);
            })
        },
        {
            let progress_bar = create_progress_bar("tailscale");
            thread::spawn(move || {
                tailscale(progress_bar);
            })
        },
        {
            let progress_bar = create_progress_bar("ping 1.1.1.1");
            thread::spawn(move || {
                ping(progress_bar, "1.1.1.1");
            })
        },
        {
            let progress_bar = create_progress_bar("ping 8.8.8.8");
            thread::spawn(move || {
                ping(progress_bar, "8.8.8.8");
            })
        },
        {
            let progress_bar = create_progress_bar("ping mensura.cdn-apple.com");
            thread::spawn(move || {
                ping(progress_bar, "mensura.cdn-apple.com");
            })
        },
        {
            let progress_bar = create_progress_bar("ping Pythagoras.tlb");
            thread::spawn(move || {
                ping(progress_bar, "Pythagoras.tlb");
            })
        },
        {
            let progress_bar = create_progress_bar("ping Pythagoras-ts.wyvern-climb.ts.net");
            thread::spawn(move || {
                ping(progress_bar, "Pythagoras-ts.wyvern-climb.ts.net");
            })
        },
        {
            let progress_bar = create_progress_bar("sshping Pythagoras.tlb");
            thread::spawn(move || {
                sshping(progress_bar, "Pythagoras.tlb");
            })
        },
        {
            let progress_bar = create_progress_bar("sshping Pythagoras-ts.wyvern-climb.ts.net");
            thread::spawn(move || {
                sshping(progress_bar, "Pythagoras-ts.wyvern-climb.ts.net");
            })
        },
        {
            let progress_bar = create_progress_bar("disk space free");
            thread::spawn(move || {
                disk_space_free(progress_bar);
            })
        },
        // Runs last
        {
            let progress_bar = create_progress_bar("networkQuality");
            sleep(Duration::from_secs(8));
            thread::spawn(move || {
                network_quality(progress_bar);
            })
        },
    ];

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

fn pythagoras_tlb_locked(progress_bar: ProgressBar) {
    let child = Command::new("ssh")
        .args(["Pythagoras.tlb", "\"is-macos-screen-locked\""])
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

fn pythagoras_ts_tailscale_locked(progress_bar: ProgressBar) {
    let child = Command::new("ssh")
        .args([
            "Pythagoras-ts.wyvern-climb.ts.net",
            "\"is-macos-screen-locked\"",
        ])
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
        .args(["-H", host])
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
