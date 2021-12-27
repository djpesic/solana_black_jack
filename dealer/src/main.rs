use black_jack_client as bj_client;

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

    let dealer = bj_client::utils::get_local_wallet().unwrap();
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

    bj_client::client::create_blackjack_account(&dealer, &program, &connection).unwrap();

    bj_client::client::send_deck(&dealer, &program, &connection).unwrap();
    println!("Dealer sent deck of cards")
}
