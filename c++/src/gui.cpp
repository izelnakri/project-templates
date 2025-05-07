#include <gtkmm.h>
#include <thread>
#include <future>
#include <sstream>
#include <nlohmann/json.hpp>
#include "user.hpp"

class GitHubUserFetcherWindow : public Gtk::Window {
public:
    GitHubUserFetcherWindow() {
        set_title("GitHub User Fetcher");
        set_default_size(400, 300);

        // Layout
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
