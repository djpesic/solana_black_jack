use black_jack_client as bj_client;
use std::process::exit;
use std::sync::mpsc::RecvTimeoutError;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() != 2 {
        eprintln!(
            "usage: {} <path to solana hello world example program keypair>",
            args[0]
        );
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

    let dealer = utils::get_local_wallet().unwrap();
    let dealer_balance = bj_client::client::get_player_balance(&dealer, &connection).unwrap();
    println!("({}) lamports are owned by dealer.", dealer_balance);

    if dealer_balance < balance_requirement {
        let request = balance_requirement - dealer_balance;
        println!(
            "dealer does not own sufficent lamports. Airdropping ({}) lamports.",
            request
        );
        bj_client::client::request_airdrop(&dealer, &connection, request).unwrap();
    }

    let program = bj_client::client::get_program(keypair_path, &connection).unwrap();

    println!("Create blackjack account");
    bj_client::client::create_blackjack_account(&dealer, &program, &connection).unwrap();
    let account_subscription =
        bj_client::client::establish_pub_sub_connection(&dealer, &program).unwrap();

    let receiver = account_subscription.1;
    let end_recv = Arc::new(Mutex::new(false));

    let end_recv1 = Arc::clone(&end_recv);
    let dealer_lock = Arc::new(Mutex::new(dealer));
    let dealer_lock1 = Arc::clone(&dealer_lock);
    let program_lock = Arc::new(Mutex::new(program));
    let program_lock1 = Arc::clone(&program_lock);
    let conn_lock = Arc::new(Mutex::new(connection));
    let conn_lock1 = Arc::clone(&conn_lock);
    let recv_thread = thread::spawn(move || loop {
        match receiver.recv_timeout(Duration::from_secs(2)) {
            Ok(val) => {
                let val = val.value;
                println!("Received event from solana network: {:?}", val);
                let account = match bj_client::client::process_solana_network_event(val) {
                    Ok(acc) => acc,
                    Err(_) => continue,
                };
                if account.last_operation == utils::REQUEST_NEW_DECK {
                    let dealer = dealer_lock1.lock().unwrap();
                    let program = program_lock1.lock().unwrap();
                    let connection = conn_lock1.lock().unwrap();
                    bj_client::actions::send_deck(&dealer, &program, &connection).unwrap();
                    println!("Dealer dealt a new deck of cards");
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

    {
        let dealer = dealer_lock.lock().unwrap();
        let program = program_lock.lock().unwrap();
        let connection = conn_lock.lock().unwrap();
        println!("Send deck of cards");
        bj_client::actions::send_deck(&dealer, &program, &connection).unwrap();
        println!("Dealer sent deck of cards");

        bj_client::actions::deal(&dealer, &program, &connection).unwrap();
        println!("Cards are dealt");
    }

    loop {
        println!("Enter option:");
        println!("1) Hit");
        println!("2) Stand");
        println!("3) Exit");
        let mut line = String::new();
        std::io::stdin().read_line(&mut line).unwrap();
        line = line.trim().to_string();
        if line == "1" {
            let dealer = dealer_lock.lock().unwrap();
            let program = program_lock.lock().unwrap();
            let connection = conn_lock.lock().unwrap();
            bj_client::actions::hit(&dealer, &program, &connection).unwrap();
        } else if line == "2" {
            let dealer = dealer_lock.lock().unwrap();
            let program = program_lock.lock().unwrap();
            let connection = conn_lock.lock().unwrap();
            bj_client::actions::stand(&dealer, &program, &connection).unwrap();
        } else if line == "3" {
            *(end_recv.lock().unwrap()) = true;
            recv_thread.join().unwrap();
            break;
        }
    }
    // must be called, because pubsubclient currently can't unsubscribe from the network.
    exit(0);
}
