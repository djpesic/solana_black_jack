use black_jack_client as bj_client;

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

    let dealer_faced_up =
        bj_client::actions::get_dealer_faced_up(&player, &program, &connection).unwrap();
}
