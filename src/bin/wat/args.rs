use clap::{CommandFactory, Parser};
use clap_complete::generator::generate;
use clap_complete::{Generator, Shell};
use std::io::stdout;
use std::process::exit;

use crate::build::CLAP_LONG_VERSION;

/// wat — tell me what's up
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[clap(name = "wat", long_version = CLAP_LONG_VERSION)]
pub struct WatArgs {
    /// Include a default selection of checks.
    #[clap(long)]
    pub default: bool,

    #[clap(long)]
    pub system: bool,

    #[clap(long)]
    pub ping: bool,

    #[clap(long)]
    pub sshping: bool,

    #[clap(long = "networkQuality")]
    pub network_quality: bool,

    #[clap(long = "networkQualitySeparately")]
    pub network_quality_separately: bool,

    #[clap(long)]
    pub iperf3: bool,

    /// Not included by default.
    #[clap(long)]
    pub iperf3_tailscale: bool,

    /// Print completions for the given shell (instead of generating any icons).
    /// These can be loaded/stored permanently (e.g. when using Homebrew), but they can also be sourced directly, e.g.:
    ///
    ///  wat --completions fish | source # fish
    ///  source <(wat --completions zsh) # zsh
    #[clap(long, verbatim_doc_comment, id = "SHELL")]
    pub completions: Option<Shell>,
}

impl WatArgs {
    pub fn include_misc(&self) -> bool {
        self.default
    }
    pub fn include_ping(&self) -> bool {
        self.default || self.ping
    }
    pub fn include_system(&self) -> bool {
        self.default || self.system
    }
    pub fn include_sshping(&self) -> bool {
        self.default || self.sshping
    }
    pub fn include_network_quality(&self) -> bool {
        self.default || self.network_quality
    }
    pub fn include_iperf3(&self) -> bool {
        self.default || self.iperf3
    }
    pub fn include_iperf3_tailscale(&self) -> bool {
        self.iperf3_tailscale
    }
    fn was_any_non_default_specified(&self) -> bool {
        self.include_misc()
            || self.include_ping()
            || self.include_system()
            || self.include_sshping()
            || self.include_network_quality()
            || self.include_iperf3()
            || self.include_iperf3_tailscale()
    }
}

fn completions_for_shell(cmd: &mut clap::Command, generator: impl Generator) {
    generate(generator, cmd, "wat", &mut stdout());
}

pub fn get_options() -> WatArgs {
    let mut command = WatArgs::command();

    let mut args = WatArgs::parse();
    if let Some(shell) = args.completions {
        completions_for_shell(&mut command, shell);
        exit(0);
    }

    if !args.was_any_non_default_specified() {
        eprintln!(
            "
Defaulting to: --all
For help, pass: --help
"
        );
        args.default = true;
    }

    args
}
