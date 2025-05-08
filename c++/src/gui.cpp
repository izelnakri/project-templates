#include <gtkmm.h>
#include <thread>
#include <future>
#include <sstream>
#include <iostream>
#include <nlohmann/json.hpp>
#include "user.hpp"

// Path to the CSS file
#ifdef NDEBUG
    // If in release mode, the CSS is in the build directory
    const std::string css_file_path = "build/style.css";
#else
    // If in debug mode, the CSS is in the source directory (for development)
    const std::string css_file_path = "src/style.css";
#endif

class GitHubUserFetcherWindow : public Gtk::Window {
public:
    GitHubUserFetcherWindow() {
        // CSS styling
        auto css_provider = Gtk::CssProvider::create();

        try {
            // Load CSS from the determined path
            css_provider->load_from_path(css_file_path);

            auto display = Gdk::Display::get_default();
            Gtk::StyleContext::add_provider_for_display(display, css_provider, GTK_STYLE_PROVIDER_PRIORITY_USER);
        } catch (const Glib::FileError& e) {
            // Handle the case where the file doesn't exist
            std::cerr << "Failed to load CSS file: " << e.what() << std::endl;
        }

        set_title("GitHub User Fetcher");
        set_default_size(400, 300);

        box.set_orientation(Gtk::Orientation::VERTICAL);
        set_child(box);

        entry.set_placeholder_text("Enter GitHub username...");
        box.append(entry);

        button.set_label("Fetch User");
        box.append(button);

        scrolled_window.set_child(text_view);
        scrolled_window.set_policy(Gtk::PolicyType::AUTOMATIC, Gtk::PolicyType::AUTOMATIC);
        scrolled_window.set_expand(true);
        box.append(scrolled_window);

        text_view.set_editable(false);
        text_view.set_wrap_mode(Gtk::WrapMode::WORD);

        button.signal_clicked().connect(sigc::mem_fun(*this, &GitHubUserFetcherWindow::on_button_clicked));
    }

private:
    Gtk::Box box;
    Gtk::Entry entry;
    Gtk::Button button;
    Gtk::ScrolledWindow scrolled_window;
    Gtk::TextView text_view;

    void on_button_clicked() {
        std::string username = entry.get_text();
        if (username.empty()) {
            show_message("Please enter a username.");
            return;
        }

        // Clear previous result
        text_view.get_buffer()->set_text("Fetching...");

        // Run fetch in a background thread
        std::thread([this, username]() {
            try {
                User user = fetch_github_user(username);
                nlohmann::json j = {
                    {"login", user.getLogin()},
                    {"name", user.getName()},
                    {"company", user.getCompany()},
                    {"location", user.getLocation()}
                };

                std::string result = j.dump(4);

                // Update UI in main thread
                Glib::signal_idle().connect_once([this, result]() {
                    text_view.get_buffer()->set_text(result);
                });

            } catch (const std::exception& e) {
                std::string error_msg = std::string("Error: ") + e.what();
                Glib::signal_idle().connect_once([this, error_msg]() {
                    text_view.get_buffer()->set_text(error_msg);
                });
            }
        }).detach();
    }

    void show_message(const std::string& msg) {
        text_view.get_buffer()->set_text(msg);
    }
};

int main(int argc, char* argv[]) {
    auto app = Gtk::Application::create("com.example.githubuserfetcher");

    app->signal_activate().connect([&]() {
        auto window = new GitHubUserFetcherWindow();
        app->add_window(*window);
        window->present();
    });

    return app->run(argc, argv);
}
