# Yolo and Solo Your Private Pool for ORE Mining

**The goal of this project is to balance the ever-growing public pools and decentralize computing power, in line with the ORE design principle: anyone can mine. As expected, more individual miners.**

**This is private pool client. Forked from [ore-hq-client](https://github.com/Kriptikz/ore-hq-client.git).**

It's tailored by Miraland Labs as a lightweight release, **derived from and credited to ore-hq-client.**

## Key Differentiators of the Private Pool

**Simplified and lightweight.**

**Optimized for home and/or personal use.**

**Zero fee charge for computing client, mining tx fee only for pool server.**

**No delegate.**

**No database.**

**Scale from a few to tens of devices, either laptop or PC.**

**Balance between worse penalties and better rewards.**

**Easy setup and flexible home deployment.**

## Install

To install the private pool client, 2 approaches are recommended:

**Approach One**: install from crates.io directly, use [cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html):

```sh
cargo install ore-private-pool-cli
```

**Approach Two**: download source code from Github at: [github](https://github.com/miraland-labs/ore-private-pool-cli):

```sh
https://github.com/miraland-labs/ore-private-pool-cli
```

and then compile locally

`cargo build --release`

### Dependencies

If you run into issues during installation, please install the following dependencies for your operating system and try again:

#### Linux

```
sudo apt-get install openssl pkg-config libssl-dev
```

#### MacOS (using [Homebrew](https://brew.sh/))

```
brew install openssl pkg-config

# If you encounter issues with OpenSSL, you might need to set the following environment variables:
export PATH="/usr/local/opt/openssl/bin:$PATH"
export LDFLAGS="-L/usr/local/opt/openssl/lib"
export CPPFLAGS="-I/usr/local/opt/openssl/include"
```

#### Windows (using [Chocolatey](https://chocolatey.org/))

```
choco install openssl pkgconfiglite
```

## Build

To build the codebase from scratch, checkout the repo and use cargo to build:

```sh
cargo build --release
```

## Run

To run pool client, execute:

```sh
ore-ppl-cli [OPTIONS]
```

or, if you build from source code downloaded from github, enter into ore-private-pool-cli home directory,
duplicate `bin.example` directory and rename to `bin`, modify settings in `start-ore-ppl-cli.sh`, execute:

```
bin/start-ore-ppl-cli.sh
```

## Help

You can use the `-h` flag on any command to pull up a help menu with documentation:

```sh
ore-ppl-cli -h

Usage: ore-ppl-cli [OPTIONS] <COMMAND>

Commands:
  mine       Connect to pool and start mining. (Default Implementation)
  protomine  Connect to pool and start mining. (Protomine Implementation)
  help       Print this message or the help of the given subcommand(s)

Options:
      --url <SERVER_URL>        Host name and port of your private pool server to connect to, it can also be your LAN ip address:port like: 172.xxx.xx.xxx:3000, 192.xxx.xx.xxx:3000 [default: orepool.miraland.io:3000]
      --keypair <KEYPAIR_PATH>  Filepath to keypair to use [default: ~/.config/solana/id.json]
  -u, --use-http                Use unsecure http connection instead of https.
```

## Support us | Donate at your discretion

We greatly appreciate any donation to help support projects development at Miraland Labs. Miraland is dedicated to freedom and individual sovereignty and we are doing our best to make it a reality.
Certainly, if you find this project helpful and would like to support its development, you can buy me/us a coffee!
Your support is greatly appreciated. It motivates me/us to keep improving this project.

**Bitcoin(BTC)**
`bc1plh7wnl0v0xfemmk395tvsu73jtt0s8l28lhhznafzrj5jwu4dy9qx2rpda`

![Donate BTC to Miraland Development](donations/donate-btc-qr-code.png)

**Solana(SOL)**
`9h9TXFtSsDAiL5kpCRZuKUxPE4Nv3W56fcSyUC3zmQip`

![Donate SOL to Miraland Development](donations/donate-sol-qr-code.png)

Thank you for your support!
