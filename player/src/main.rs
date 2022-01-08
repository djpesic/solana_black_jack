extern crate std_semaphore;

use black_jack_client as bj_client;
use std::process::exit;
use std::sync::mpsc::RecvTimeoutError;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use std_semaphore::Semaphore;

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() != 2 {
        eprintln!("usage: {} <path to program keypair>", args[0]);
        std::process::exit(-1);
    }
    let keypair_path = &args[1];

    let connection = bj_client::client::establish_connection().unwrap();
    println!(
        "Connected to remote solana node running version ({}).",
        connection.get_version().unwrap()
    );

    let balance_requirement = bj_client::client::get_balance_requirement(&connection).unwrap();
    println!(
        "({}) lamports are required for this transaction.",
        balance_requirement
    );

    let player = utils::get_local_wallet().unwrap();
    let player_balance = bj_client::client::get_player_balance(&player, &connection).unwrap();
    println!("({}) lamports are owned by player.", player_balance);

    if player_balance < balance_requirement {
        let request = balance_requirement - player_balance;
        println!(
            "player does not own sufficent lamports. Airdropping ({}) lamports.",
            request
        );
        bj_client::client::request_airdrop(&player, &connection, request).unwrap();
    }

    let program = bj_client::client::get_program(keypair_path, &connection).unwrap();

    let account_subscription =
        bj_client::client::establish_pub_sub_connection(&player, &program).unwrap();

    let receiver = account_subscription.1;
    let end_recv = Arc::new(Mutex::new(false));
    let end_recv1 = Arc::clone(&end_recv);

    let deck_created = Arc::new(Semaphore::new(0));
    let deck_created1 = Arc::clone(&deck_created);

    let recv_thread = thread::spawn(move || loop {
        match receiver.recv_timeout(Duration::from_secs(2)) {
            Ok(val) => {
                let val = val.value;
                println!("Received event from solana network: {:?}", val);
                let account_data = bj_client::client::process_solana_network_event(val).unwrap();
                if account_data.last_operation == utils::DEAL {
                    (*deck_created1).release();
                }
            }
            Err(RecvTimeoutError::Timeout) => {
                let should_finish = end_recv1.lock().unwrap();
                if *should_finish {
                    println!("Receiver ended properly.");
                    return;
                }
            }
            Err(RecvTimeoutError::Disconnected) => {
                println!("Received disconnected");
                return;
            }
        }
    });
    if !bj_client::actions::is_deck_dealt(&player, &program, &connection).unwrap() {
        println!("Waiting for dealer do create the deck");
        (*deck_created).acquire();
    }
    println!("Cards are dealt, now game can begin");
    bj_client::actions::get_init_status(&player, &program, &connection).unwrap();

    loop {
        println!("Enter option:");
        println!("1) Hit");
        println!("2) Stand");
        println!("3) Exit");
        let mut line = String::new();
        std::io::stdin().read_line(&mut line).unwrap();
        line = line.trim().to_string();
        if line == "1" {
            bj_client::actions::hit(&player, &program, &connection).unwrap();
        } else if line == "2" {
            bj_client::actions::stand(&player, &program, &connection).unwrap();
        } else if line == "3" {
            *(end_recv.lock().unwrap()) = true;
            recv_thread.join().unwrap();
            bj_client::actions::clear_data(&player, &program, &connection).unwrap();
            break;
        }
    }
    // must be called, because pubsubclient currently can't unsubscribe from the network.
    exit(0);
}
