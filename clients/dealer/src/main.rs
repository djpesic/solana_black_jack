use client as bj_client;
use std::process::exit;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use std_semaphore::Semaphore;
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

    let wait_player = Arc::new(Semaphore::new(0));
    let wait_player1 = Arc::clone(&wait_player);

    let is_busted = Arc::new(Mutex::new(false));
    let is_busted1 = Arc::clone(&is_busted);

    let hit_sem = Arc::new(Semaphore::new(0));
    let hit_sem1 = Arc::clone(&hit_sem);

    let last_player_hand = Arc::new(Mutex::new(0));
    let last_player_hand1 = Arc::clone(&last_player_hand);

    let dealer_hand = Arc::new(Mutex::new(0));
    let dealer_hand1 = Arc::clone(&dealer_hand);

    let recv_thread = thread::spawn(move || loop {
        match receiver.recv_timeout(Duration::from_secs(2)) {
            Ok(val) => {
                let val = val.value;
                // println!("Received event from solana network: {:?}", val);
                let account_data = match bj_client::client::process_solana_network_event(val) {
                    Ok(acc) => acc,
                    Err(_) => continue,
                };
                if account_data.last_operation == utils::REQUEST_NEW_DECK {
                    let dealer = dealer_lock1.lock().unwrap();
                    let program = program_lock1.lock().unwrap();
                    let connection = conn_lock1.lock().unwrap();
                    bj_client::actions::send_deck(&dealer, &program, &connection).unwrap();
                    println!("Dealer dealt a new deck of cards");
                    bj_client::actions::deal(&dealer, &program, &connection).unwrap();
                    println!("New cards are dealt, waiting for player to finish");
                } else if account_data.last_operation == utils::PLAYER_BUSTED {
                    *is_busted1.lock().unwrap() = true;
                    wait_player1.release();
                } else if account_data.last_operation == utils::PLAYER_STAND {
                    println!("Player stands with {}", account_data.player_hand);
                    println!("Sum of dealer current hand is {}", account_data.dealer_hand);
                    *last_player_hand1.lock().unwrap() = account_data.player_hand;
                    *dealer_hand1.lock().unwrap() = account_data.dealer_hand;
                    wait_player1.release();
                } else if account_data.last_operation == utils::DEALER_HIT {
                    println!("Sum of current dealer hand is {}", account_data.dealer_hand);
                    *dealer_hand1.lock().unwrap() = account_data.dealer_hand;
                    if account_data.dealer_hand > 21 {
                        *is_busted1.lock().unwrap() = true;
                    }
                    hit_sem1.release();
                }
            }
            Err(crossbeam_channel::RecvTimeoutError::Timeout) => {
                let should_finish = end_recv1.lock().unwrap();
                if *should_finish {
                    println!("Receiver ended properly.");
                    return;
                }
            }
            Err(crossbeam_channel::RecvTimeoutError::Disconnected) => {
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
        println!("Cards are dealt, waiting for player to finish");
    }
    wait_player.acquire();
    if *is_busted.lock().unwrap() {
        println!("Player busted, dealer wins.");
        *(end_recv.lock().unwrap()) = true;
        recv_thread.join().unwrap();
        let dealer = dealer_lock.lock().unwrap();
        let program = program_lock.lock().unwrap();
        let connection = conn_lock.lock().unwrap();
        bj_client::actions::clear_data(&dealer, &program, &connection).unwrap();
        // must be called, because pubsubclient currently can't unsubscribe from the network.
        exit(0);
    }

    println!("Delaer should hit until beats player, or go busted");
    loop {
        println!("Enter option:");
        println!("1) Hit");
        println!("2) Stand");
        let mut line = String::new();
        std::io::stdin().read_line(&mut line).unwrap();
        line = line.trim().to_string();
        if line == "1" {
            let dealer = dealer_lock.lock().unwrap();
            let program = program_lock.lock().unwrap();
            let connection = conn_lock.lock().unwrap();
            bj_client::actions::hit(&dealer, &program, &connection, utils::DEALER_HIT).unwrap();
            hit_sem.acquire();
            if *is_busted.lock().unwrap() {
                println!("DEALER BUSTED");
                //notify player and finish
                bj_client::actions::busted(&dealer, &program, &connection, utils::DEALER_BUSTED)
                    .unwrap();
                break;
            }
        } else if line == "2" {
            if *dealer_hand.lock().unwrap() > *last_player_hand.lock().unwrap() {
                let dealer = dealer_lock.lock().unwrap();
                let program = program_lock.lock().unwrap();
                let connection = conn_lock.lock().unwrap();
                bj_client::actions::stand(&dealer, &program, &connection, utils::DEALER_STAND)
                    .unwrap();
                break;
            } else {
                println!("Dealer should continue hitting");
            }
        }
    }
    *(end_recv.lock().unwrap()) = true;
    recv_thread.join().unwrap();
    let dealer = dealer_lock.lock().unwrap();
    let program = program_lock.lock().unwrap();
    let connection = conn_lock.lock().unwrap();
    bj_client::actions::clear_data(&dealer, &program, &connection).unwrap();
    // must be called, because pubsubclient currently can't unsubscribe from the network.
    exit(0);
}
