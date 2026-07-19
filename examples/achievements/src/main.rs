use std::io::{self, Write};

use steamworks::{steam_api_exists, AppId, CallbackResult, Client};

// 480 is Spacewar!, the official example game included with the Steamworks SDK.
// Steamworks predefines a set of achievements for Spacewar!; all of their API names are listed below.
const SPACEWAR_APP_ID: u32 = 480;

const SPACEWAR_ACHIEVEMENTS: &[&str] = &[
    "ACH_WIN_ONE_GAME",
    "ACH_WIN_100_GAMES",
    "ACH_TRAVEL_FAR_ACCUM",
    "ACH_TRAVEL_FAR_SINGLE",
    "NEW_ACHIEVEMENT_0_4",
];

fn print_achievement_status(client: &Client, name: &str) {
    let user_stats = client.user_stats();
    let achievement = user_stats.achievement(name);

    let achieved = achievement.get();
    match achieved {
        Ok(true) => {
            let (_, unlock_time) = achievement
                .get_achievement_and_unlock_time()
                .expect("Failed to get achievement unlock time");
            println!("  [x] {name}  (unlocked, unlock time: {unlock_time})");
        }
        Ok(false) => {
            println!("  [ ] {name}  (locked)");
        }
        Err(()) => {
            println!("  [?] {name}  (unable to read; the achievement API name may not exist)");
        }
    }
}

fn list_achievements(client: &Client) {
    println!("\n=== Current Spacewar! Achievement Status ===");
    for name in SPACEWAR_ACHIEVEMENTS {
        print_achievement_status(client, name);
    }
    println!();
}

fn set_achievement(client: &Client, name: &str) {
    let user_stats = client.user_stats();
    let achievement = user_stats.achievement(name);

    match achievement.set() {
        Ok(()) => {
            // set() only changes the in-memory state. Call store_stats() to sync it
            // with the Steam server and trigger an unlock notification in the Steam overlay.
            match user_stats.store_stats() {
                Ok(()) => println!("Achievement {name} unlocked and synced to the Steam server."),
                Err(()) => {
                    println!("Achievement {name} unlocked in memory, but store_stats failed.")
                }
            }
        }
        Err(()) => {
            println!("Failed to unlock achievement {name}; verify that its API name exists in Spacewar!.");
        }
    }
}

fn clear_achievement(client: &Client, name: &str) {
    let user_stats = client.user_stats();
    let achievement = user_stats.achievement(name);

    match achievement.clear() {
        Ok(()) => match user_stats.store_stats() {
            Ok(()) => println!("Achievement {name} reset and synced to the Steam server."),
            Err(()) => println!("Achievement {name} reset in memory, but store_stats failed."),
        },
        Err(()) => {
            println!(
                "Failed to reset achievement {name}; verify that its API name exists in Spacewar!."
            );
        }
    }
}

fn print_help() {
    println!("\nAvailable commands:");
    println!("  list                List all Spacewar! achievements and their unlock status");
    println!(
        "  unlock <name>       Unlock the specified achievement (e.g. unlock ACH_WIN_THE_GAME)"
    );
    println!("  unlockall           Unlock all achievements");
    println!("  reset <name>        Reset the specified achievement");
    println!("  resetall            Reset all achievements");
    println!("  help                Show this help");
    println!("  quit                Exit the program");
}

fn main() {
    if !steam_api_exists() {
        eprintln!("Failed to load the Steam API dynamic library.");
        return;
    }

    // Explicitly specify Spacewar!'s AppId with init_app, so steam_appid.txt
    // is not required in the working directory.
    let client = Client::init_app(AppId(SPACEWAR_APP_ID))
        .expect("The Steam client is not running or the Steamworks API could not be initialized");

    println!("Connected to Steam. Using Spacewar! (AppId {SPACEWAR_APP_ID}) as the example game.");
    println!(
        "Currently logged-in Steam user: {}",
        client.friends().get_friend(client.user().steam_id()).name()
    );

    list_achievements(&client);
    print_help();

    let stdin = io::stdin();
    loop {
        // Continuously process callbacks so events such as UserAchievementStored
        // and UserStatsStored can be dispatched.
        client.process_callbacks(|event| {
            if let CallbackResult::UserAchievementStored(stored) = event {
                println!(
                    "\n[Callback] Achievement stored: {} (progress {}/{})",
                    stored.achievement_name, stored.current_progress, stored.max_progress
                );
            } else if let CallbackResult::UserStatsStored(stored) = event {
                if stored.result.is_ok() {
                    println!("\n[Callback] Statistics successfully synced to the Steam server.");
                } else {
                    println!(
                        "\n[Callback] Failed to sync statistics: {:?}",
                        stored.result
                    );
                }
            }
        });

        print!("> ");
        io::stdout().flush().ok();

        let mut input = String::new();
        if stdin.read_line(&mut input).is_err() {
            break;
        }
        let input = input.trim();
        if input.is_empty() {
            continue;
        }

        let mut parts = input.split_whitespace();
        let command = parts.next().unwrap_or("");
        let arg = parts.next();

        match command {
            "list" => list_achievements(&client),
            "unlock" => match arg {
                Some(name) => set_achievement(&client, name),
                None => println!("Usage: unlock <achievement API name>"),
            },
            "unlockall" => {
                for name in SPACEWAR_ACHIEVEMENTS {
                    set_achievement(&client, name);
                }
                println!("All achievements unlocked.");
            }
            "reset" => match arg {
                Some(name) => clear_achievement(&client, name),
                None => println!("Usage: reset <achievement API name>"),
            },
            "resetall" => {
                for name in SPACEWAR_ACHIEVEMENTS {
                    clear_achievement(&client, name);
                }
                println!("All achievements reset.");
            }
            "help" => print_help(),
            "quit" | "exit" => break,
            other => println!("Unknown command: {other} (enter help to see available commands)"),
        }
    }

    println!("Goodbye!");
}
