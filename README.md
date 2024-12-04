# wat

System is acting weird → `wat` → find out `wat` is going on!

*Extremely* hardcoded to my own setup, but should be easy to hack.

## Installation

```
cargo install faketty sshping
brew install --HEAD lgarron/lgarron/wat
```

## Example

```
> wat

Defaulting to: --all
For help, pass: --help

    0s | Charging | 🔌 140 watts
    0s | 🔥 thermal pressure | Nominal
    0s | 💾 disk space free | 244Gi
    0s | 🔈 audio output | USB Audio Device
    0s | IPv4 address (WiFi) | ###.###.###.### (en0)
    0s | IPv4 address (other) | ###.###.###.### (en17)
    1s | tailscale | ↔️ up
    0s | Pythagoras.tlb locked? | 🔒 locked
    0s | Pythagoras-ts.wyvern-climb.ts.net locked? | 🔒 locked
    7s | ping 1.1.1.1 | round-trip min/avg/max/stddev = 13.649/20.987/56.322/13.560 ms
    7s | ping 8.8.8.8 | round-trip min/avg/max/stddev = 11.211/17.308/41.737/9.493 ms
    7s | ping mensura.cdn-apple.com | round-trip min/avg/max/stddev = 13.902/15.008/16.499/0.892 ms
    7s | ping Pythagoras.tlb | round-trip min/avg/max/stddev = 0.318/4.643/33.304/10.835 ms
    7s | ping Pythagoras-ts.wyvern-climb.ts.net | round-trip min/avg/max/stddev = 0.850/9.610/53.542/16.881 ms
    4s | sshping Pythagoras.tlb | Ping 979/0:                    277 us
    4s | sshping Pythagoras-ts.wyvern-climb.ts.net | Ping 1004/0:                   723 us
   12s | networkQuality | Downlink: 660.958 Mbps, 614 RPM - Uplink: 35.757 Mbps, 614 RPM
   10s | Pythagoras.tlb iperf3 | [  7]   0.00-10.00  sec  6.84 GBytes  5.87 Gbits/sec                  receiver
```

## Usage

````cli-help
wat — tell me what's up

Usage: wat [OPTIONS]

Options:
      --default
          Include a default selection of checks

      --system
          

      --ping
          

      --sshping
          

      --networkQuality
          

      --iperf3
          

      --iperf3-tailscale
          Not included by default

      --completions <SHELL>
          Print completions for the given shell (instead of generating any icons).
          These can be loaded/stored permanently (e.g. when using Homebrew), but they can also be sourced directly, e.g.:
          
           wat --completions fish | source # fish
           source <(wat --completions zsh) # zsh
          
          [possible values: bash, elvish, fish, powershell, zsh]

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
````
