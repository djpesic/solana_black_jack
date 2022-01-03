# Decentralized black jack game
This is decentralized implementation of black jack game, implemented on Solana, in Rust. Currently, game supports one deck of cards, one player and a dealer. Code is based on https://github.com/ezekiiel/simple-solana-program.git. There are two main parts:

 1. Clients
 2. Solana program.

### Clients
### Solana program

## Prerequisites
* Install Rust: https://doc.rust-lang.org/book/ch01-01-installation.html
* Install Solana: https://docs.solana.com/cli/install-solana-cli-tools
* Development is done on Linux (Ubuntu 18.04). 
If you don't have local solana setup, run the following commands to configure you machine for local development:
```
solana config set --url localhost
solana-keygen new
```

These two commands create Solana config files in  `~/.config/solana/`  which solana command line tools will read in to determine what cluster to connect to and what keypair to use.
Having done that run a local Solana validator by running:
```
solana-test-validator
```
This program must be left running in the background.

 ## How to build and run
- Download source code
 `git@gitlab.com:Pesic/rust_solana_blackjack.git`
- Compile source code
`./run.sh build`
- Application currently works only on local test blockchain.
- Deploy commands and keypair location are displayed at the end of the build output. Also, you can use this command: `/run.sh deploy`
- Cleanup build: `./run.sh clean`
- Run client: TBD