mod user; // Assuming user module contains fetch_github_user and User struct

use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Button, Entry, TextBuffer, TextView, Box as GtkBox, Orientation};
use glib::clone;
use glib::MainContext; // No longer need `clone` directly from here, use glib::clone!
use reqwest::Client;
use std::sync::OnceLock;
use tokio::runtime::Runtime;
use async_channel::bounded;
use std::error::Error; // Import the standard Error trait for the boxed error type

// Initialize a Tokio runtime once.
fn runtime() -> &'static Runtime {
    static RUNTIME: OnceLock<Runtime> = OnceLock::new();
    RUNTIME.get_or_init(|| {
        Runtime::new().expect("Failed to create Tokio runtime")
    })
}

pub fn build_ui(app: &Application, client: Client) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("GitHub User Fetcher")
        .default_width(400)
        .default_height(300)
        .build();
    let entry = Entry::builder()
        .placeholder_text("Enter GitHub username")
        .build();
    let button = Button::with_label("Fetch User");
    // Output Text View (non-editable)
    let buffer = TextBuffer::new(None);
    let text_view = TextView::builder()
        .buffer(&buffer)
        .editable(false)
        .build();
    let layout = GtkBox::new(Orientation::Vertical, 5);

    layout.set_css_classes(&["margin-10"]); // Add some margin (requires GTK CSS provider)
    layout.append(&entry);
    layout.append(&button);
    layout.append(&text_view);
    window.set_child(Some(&layout));
    window.show();

    // Setup async channel for communication between Tokio task and GTK main thread
    // The channel will carry the Result with the boxed error type that
    // fetch_github_user is currently returning.
    let (sender, receiver) = bounded::<Result<user::User, Box<dyn Error + Send + Sync>>>(1);

    // NOTE: check if this can be simplified
    button.connect_clicked(clone!(
        #[strong] entry, 
        #[strong] buffer, 
        #[strong] sender, 
        #[strong] client,
        move |_| {
            let username = entry.text().trim().to_string();
            if username.is_empty() {
                buffer.set_text("Please enter a GitHub username.");
                return;
            }

            buffer.set_text("Fetching user data...");

            // Spawn the async task on the Tokio runtime
            runtime().spawn(clone!(
                #[strong] sender, 
                #[strong] client,
                async move {
                    // Perform the potentially long-running async operation
                    // The type annotation confirms we expect the boxed error type here
                    let result = user::fetch_github_user(&client, &username).await;

                    // Send the result back to the GTK main thread via the channel
                    // We check the send result to catch potential channel errors (e.g., receiver dropped)
                    if let Err(e) = sender.send(result).await {
                        eprintln!("Failed to send result to GTK thread: {}", e);
                    }
                }
            ));
        }
    ));

    // Receive result on the GTK main thread and update UI
    // This loop runs on the GTK main context and waits for messages from the channel.
    MainContext::default().spawn_local(clone!(
        #[strong] buffer,
        async move {
            while let Ok(result) = receiver.recv().await {
                match result {
                    Ok(user) => {
                        // Successfully fetched user data, serialize and display it
                        match serde_json::to_string_pretty(&user) {
                            Ok(json) => buffer.set_text(&json),
                            Err(e) => {
                                // Handle serialization errors
                                buffer.set_text(&format!("Serialization error: {e}"));
                                eprintln!("Serialization error: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        // Handle fetch errors (e.g., network issues, user not found)
                        // Format the boxed error for display
                        buffer.set_text(&format!("Error: {}", e));
                        eprintln!("Fetch error: {}", e);
                    }
                }
            }
            // This part is reached if the sender is dropped, which might indicate an issue
            eprintln!("Channel receiver loop finished. Sender was likely dropped.");
        }
    ));
}

fn main() {
    let app = Application::builder()
        .application_id("com.example.GitHubUserFetcher")
        .build();
    // Connect the activate signal to build the UI
    app.connect_activate(move |app| {
        build_ui(app, Client::builder()
            .user_agent("rust-poem-github-server")
            .build()
            .expect("Failed to build reqwest client"));
    });

    // Run the GTK application. This will block until the application exits.
    app.run();
}
