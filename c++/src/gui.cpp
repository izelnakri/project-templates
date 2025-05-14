/**
 * @file gui.cpp
 * @brief GUI application for fetching and displaying GitHub user information.
 *
 * This file defines a GTK-based graphical user interface that allows users
 * to input a GitHub username, fetch user information via HTTP, and display it
 * as formatted JSON. The GUI is styled with embedded CSS.
 */

#include "user.hpp"
#include <future>
#include <gtkmm.h>
#include <iostream>
#include <nlohmann/json.hpp>
#include <sstream>
#include <thread>

#include "style_css.hpp" // cppcheck-suppress missingInclude // NOLINT(clang-diagnostic-error)

// Path to the CSS file
/*#ifdef NDEBUG*/
/*    // If in release mode, the CSS is in the build directory*/
/*    const std::string css_file_path = "build/style.css";*/
/*#else*/
/*    // If in debug mode, the CSS is in the source directory (for
 * development)*/
/*    const std::string css_file_path = "src/style.css";*/
/*#endif*/

/**
 * @class GitHubUserFetcherWindow
 * @brief Main application window for the GitHub user fetcher GUI.
 *
 * This class sets up the UI elements such as the input field, button,
 * and display area. It fetches user data from GitHub in a background
 * thread and updates the UI asynchronously using Glib's main loop.
 */
class GitHubUserFetcherWindow : public Gtk::Window {
public:
  /**
   * @brief Constructs the main window, sets up widgets and signals.
   *
   * Applies CSS styling, initializes all widgets, and connects the button
   * click signal to the event handler.
   */
  GitHubUserFetcherWindow() {
    auto css_provider = Gtk::CssProvider::create();
    try {
      css_provider->load_from_data(css_data);
      auto display = Gdk::Display::get_default();
      Gtk::StyleContext::add_provider_for_display(
          display, css_provider, GTK_STYLE_PROVIDER_PRIORITY_USER);
    } catch (const Glib::FileError &e) {
      std::cerr << "Failed to load CSS data: " << e.what() << '\n';
    } catch (...) {
      std::cerr << "Unknown error occurred while loading CSS.\n";
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
    scrolled_window.set_policy(Gtk::PolicyType::AUTOMATIC,
                               Gtk::PolicyType::AUTOMATIC);
    scrolled_window.set_expand(true);
    box.append(scrolled_window);

    text_view.set_editable(false);
    text_view.set_wrap_mode(Gtk::WrapMode::WORD);

    button.signal_clicked().connect(
        sigc::mem_fun(*this, &GitHubUserFetcherWindow::on_button_clicked));
  }

private:
  Gtk::Box box;                        ///< Container for layout
  Gtk::Entry entry;                    ///< Text field for GitHub username input
  Gtk::Button button;                  ///< Button to trigger fetch
  Gtk::ScrolledWindow scrolled_window; ///< Scrollable container for text_view
  Gtk::TextView text_view; ///< Text view to display results or messages

  /**
   * @brief Event handler for the fetch button.
   *
   * Starts a background thread to fetch GitHub user data and updates the
   * text view with the results. Displays an error message on failure.
   */
  void on_button_clicked() {
    std::string username = entry.get_text();
    if (username.empty()) {
      show_message("Please enter a username.");
      return;
    }

    text_view.get_buffer()->set_text("Fetching...");

    std::thread([this, username]() {
      try {
        User user = User::fetch_github_user(username);
        nlohmann::json json = {{"login", user.login},
                               {"name", user.name},
                               {"company", user.company},
                               {"location", user.location}};

        std::string result = json.dump(4);

        Glib::signal_idle().connect_once(
            [this, result]() { text_view.get_buffer()->set_text(result); });

      } catch (const std::exception &e) {
        std::string error_msg = std::string("Error: ") + e.what();
        Glib::signal_idle().connect_once([this, error_msg]() {
          text_view.get_buffer()->set_text(error_msg);
        });
      }
    }).detach();
  }

  /**
   * @brief Displays a message in the text view area.
   * @param msg The message to display.
   */
  void show_message(const std::string &msg) {
    text_view.get_buffer()->set_text(msg);
  }
};

/**
 * @brief Application entry point for the GUI.
 *
 * Initializes and runs the GTK application, creating a single main window.
 *
 * @param argc Argument count.
 * @param argv Argument vector.
 * @return Exit code.
 */
int main(int argc, char *argv[]) { // NOLINT(cppcoreguidelines-avoid-c-arrays)
  auto app = Gtk::Application::create("com.example.githubuserfetcher");

  app->signal_activate().connect([&]() {
    static GitHubUserFetcherWindow window;
    app->add_window(window);
    window.present();
  });

  return app->run(argc, argv);
}
