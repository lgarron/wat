use clap::{CommandFactory, Parser};
use clap_complete::generator::generate;
use clap_complete::{Generator, Shell};
use std::io::stdout;
use std::process::exit;

/// twsearch — solve every puzzle.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[clap(name = "wat")]
pub struct WatArgs {
    #[clap(long)]
    pub all: bool,

    #[clap(long)]
    pub system: bool,

    #[clap(long)]
    pub ping: bool,

    #[clap(long)]
    pub sshping: bool,

    #[clap(long)]
    pub iperf3: bool,

    #[clap(long = "networkQuality")]
    pub network_quality: bool,

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
        self.all
    }
    pub fn include_ping(&self) -> bool {
        self.all || self.ping
    }
    pub fn include_system(&self) -> bool {
        self.all || self.system
    }
    pub fn include_sshping(&self) -> bool {
        self.all || self.sshping
    }
    pub fn include_network_quality(&self) -> bool {
        self.all || self.network_quality
    }
    pub fn include_iperf3(&self) -> bool {
        self.all || self.iperf3
    }
    fn include_any(&self) -> bool {
        self.all || self.ping || self.system || self.sshping || self.network_quality || self.iperf3
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

    if !args.include_any() {
        eprintln!(
            "
Defaulting to: --all
For help, pass: --help
"
        );
        args.all = true;
    }

    args
}
