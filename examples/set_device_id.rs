use std::io::{self, Write};
use std::path::Path;
use webull_unofficial::{error::Result, utils::save_did, LiveWebullClient};

fn main() -> Result<()> {
    println!("=====================================");
    println!("    Webull Device ID Manager         ");
    println!("=====================================\n");

    println!("This program helps you set and save a device ID for Webull API access.\n");
    println!("Device IDs are used to identify your client to Webull's servers.\n");

    // Create a live client (device ID management is directly on LiveWebullClient)
    // Both Live and Paper clients use the same device ID mechanism through the base client
    let mut client = LiveWebullClient::new(Some(6))?;

    // Display current device ID
    println!("Current Device ID: {}", client.get_did());
    println!("(This was auto-generated or loaded from saved file)\n");

    // Show menu
    loop {
        println!("=====================================");
        println!("         Device ID Options           ");
        println!("=====================================");
        println!("1. Display current device ID");
        println!("2. Set custom device ID");
        println!("3. Save current device ID to file");
        println!("4. Load device ID from file");
        println!("0. Exit");
        println!("=====================================");

        let choice = get_user_input("Enter your choice: ");

        match choice.trim() {
            "1" => {
                display_current_did(&client);
            }
            "2" => {
                set_custom_did(&mut client)?;
            }
            "3" => {
                save_current_did(&client)?;
            }
            "4" => {
                load_did_from_file(&mut client)?;
            }
            "0" | "q" | "Q" => {
                println!("\n👋 Goodbye!");
                break;
            }
            _ => {
                println!("❌ Invalid choice. Please try again.\n");
            }
        }

        if choice.trim() != "0" && choice.trim() != "q" && choice.trim() != "Q" {
            println!("\nPress Enter to continue...");
            let _ = get_user_input("");
        }
    }

    Ok(())
}

fn display_current_did(client: &LiveWebullClient) {
    println!("\n📋 Current Device ID Information");
    println!("────────────────────────────────");
    let did = client.get_did();
    println!("Device ID: {}", did);
    println!("Length: {} characters", did.len());

    // Show first and last few characters for easy identification
    if did.len() >= 8 {
        println!("Preview: {}...{}", &did[..4], &did[did.len() - 4..]);
    }
}

fn set_custom_did(client: &mut LiveWebullClient) -> Result<()> {
    println!("\n✏️  Set Custom Device ID");
    println!("────────────────────────");
    println!("Enter your custom device ID:\n");

    let custom_did = get_user_input("Enter device ID: ").to_lowercase();

    println!("\nDevice ID: {}", custom_did);
    if custom_did.len() >= 8 {
        println!(
            "Preview: {}...{}",
            &custom_did[..4],
            &custom_did[custom_did.len() - 4..]
        );
    }

    if confirm_action("Set this as your device ID?") {
        // Ask where to save
        println!("\nSave to file?");
        if confirm_action("Save device ID to default location?") {
            client.set_did(&custom_did, None)?;
            save_did(&custom_did, None)?;
            println!("✅ Device ID set and saved to did.bin");
        } else {
            // Update in memory only - don't save to file
            client.set_did(&custom_did, Some(Path::new("/tmp/.webull_did_temp")))?;
            println!("✅ Device ID set (not saved to file)");
        }
    } else {
        println!("Device ID not changed.");
    }

    Ok(())
}

fn save_current_did(client: &LiveWebullClient) -> Result<()> {
    println!("\n💾 Save Current Device ID");
    println!("─────────────────────────");

    let did = client.get_did();
    println!(
        "Current Device ID: {}...{}",
        &did[..4],
        &did[did.len() - 4..]
    );

    let path = get_user_input("Enter file path (or press Enter for default): ");

    if path.trim().is_empty() {
        // Use default path (did.bin in current directory)
        save_did(did, None)?;
        println!("✅ Device ID saved to: did.bin");
    } else {
        // Use custom path
        save_did(did, Some(Path::new(&path)))?;
        println!("✅ Device ID saved to: {}", path);
    }

    Ok(())
}

fn load_did_from_file(client: &mut LiveWebullClient) -> Result<()> {
    println!("\n📂 Load Device ID from File");
    println!("───────────────────────────");

    let path = get_user_input("Enter file path: ");

    match std::fs::read_to_string(&path) {
        Ok(did) => {
            let did = did.trim();

            if did.len() >= 8 {
                println!("Found Device ID: {}...{}", &did[..4], &did[did.len() - 4..]);
            } else {
                println!("Found Device ID: {}", did);
            }

            if confirm_action("Load this device ID?") {
                client.set_did(did, None)?;
                // Also save to default location for consistency
                save_did(did, None)?;
                println!("✅ Device ID loaded and saved to did.bin");
            } else {
                println!("Device ID not changed.");
            }
        }
        Err(e) => {
            println!("❌ Failed to read file: {}", e);
        }
    }

    Ok(())
}

fn get_user_input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

fn confirm_action(prompt: &str) -> bool {
    let response = get_user_input(&format!("❓ {} (y/n): ", prompt));
    response.to_lowercase() == "y" || response.to_lowercase() == "yes"
}
