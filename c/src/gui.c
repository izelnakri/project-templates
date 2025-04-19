#include <gtk/gtk.h>

typedef struct {
    GtkEntry *entry;
    GtkTextBuffer *output_buffer;
} AppWidgets;

static void display_output(GtkTextBuffer *buffer, const gchar *output) {
    gtk_text_buffer_set_text(buffer, output, -1);
}

static void on_button_clicked(GtkButton *button, gpointer user_data) {
    AppWidgets *widgets = user_data;
    const gchar *username = gtk_editable_get_text(GTK_EDITABLE(widgets->entry));

    if (!username || *username == '\0') {
        display_output(widgets->output_buffer, "Please enter a username.");
        return;
    }

    gchar *argv[] = {
        "/home/izelnakri/Github/c-projects/c/build/github_user_fetcher",
        (gchar *)username,
        NULL
    };

    gchar *stdout_data = NULL;
    gchar *stderr_data = NULL;
    GError *error = NULL;
    gint exit_status;

    gboolean success = g_spawn_sync(
        NULL, argv, NULL,
        G_SPAWN_DEFAULT, NULL, NULL,
        &stdout_data, &stderr_data,
        &exit_status, &error
    );

    if (!success) {
        gchar *msg = g_strdup_printf("Error: %s", error->message);
        display_output(widgets->output_buffer, msg);
        g_free(msg);
        g_error_free(error);
        return;
    }

    if (exit_status != 0) {
        gchar *msg = g_strdup_printf("Process failed: %s", stderr_data);
        display_output(widgets->output_buffer, msg);
        g_free(msg);
    } else {
        display_output(widgets->output_buffer, stdout_data);
    }

    g_free(stdout_data);
    g_free(stderr_data);
}

static void activate(GtkApplication *app, gpointer user_data) {
    GtkWidget *window = gtk_application_window_new(app);
    gtk_window_set_title(GTK_WINDOW(window), "GitHub User Fetcher");
    gtk_window_set_default_size(GTK_WINDOW(window), 600, 400);

//     GtkCssProvider *provider = gtk_css_provider_new();
//     gtk_css_provider_load_from_path(provider, "style.css");
//     gtk_style_context_add_provider_for_display(
//         gdk_display_get_default(),
//         GTK_STYLE_PROVIDER(provider),
//         GTK_STYLE_PROVIDER_PRIORITY_APPLICATION
//     );

    GtkWidget *box = gtk_box_new(GTK_ORIENTATION_VERTICAL, 10);
    gtk_widget_add_css_class(box, "main-container");
    // styling:
    gtk_widget_set_margin_top(box, 20);
    gtk_widget_set_margin_bottom(box, 20);
    gtk_widget_set_margin_start(box, 20);
    gtk_widget_set_margin_end(box, 20);
    // styling end
    gtk_window_set_child(GTK_WINDOW(window), box);

    AppWidgets *widgets = g_new0(AppWidgets, 1);

    widgets->entry = GTK_ENTRY(gtk_entry_new());
    gtk_entry_set_placeholder_text(widgets->entry, "Enter GitHub username");
    gtk_widget_add_css_class(GTK_WIDGET(widgets->entry), "username-entry");
    gtk_box_append(GTK_BOX(box), GTK_WIDGET(widgets->entry));

    GtkWidget *button = gtk_button_new_with_label("Fetch");
    gtk_widget_add_css_class(button, "fetch-button");
    gtk_box_append(GTK_BOX(box), button);

    GtkWidget *scrolled = gtk_scrolled_window_new();
    gtk_widget_set_vexpand(scrolled, TRUE);
    gtk_box_append(GTK_BOX(box), scrolled);

    GtkWidget *output_view = gtk_text_view_new();
    gtk_text_view_set_editable(GTK_TEXT_VIEW(output_view), FALSE);
    gtk_text_view_set_wrap_mode(GTK_TEXT_VIEW(output_view), GTK_WRAP_WORD_CHAR);
    gtk_widget_add_css_class(output_view, "output-view");

    gtk_scrolled_window_set_child(GTK_SCROLLED_WINDOW(scrolled), output_view);

    widgets->output_buffer = gtk_text_view_get_buffer(GTK_TEXT_VIEW(output_view));

    g_signal_connect(button, "clicked", G_CALLBACK(on_button_clicked), widgets);
    g_signal_connect(window, "close-request", G_CALLBACK(gtk_window_destroy), NULL);

    gtk_window_present(GTK_WINDOW(window));
}

int main(int argc, char *argv[]) {
    GtkApplication *app = gtk_application_new("com.example.githubfetcher", G_APPLICATION_FLAGS_NONE);
    g_signal_connect(app, "activate", G_CALLBACK(activate), NULL);
    int status = g_application_run(G_APPLICATION(app), argc, argv);
    g_object_unref(app);
    return status;
}


// ======== LAST ONE:
// #include <gtk/gtk.h>
//
// typedef struct {
//     GtkEntry *entry;
//     GtkTextBuffer *output_buffer;
//     GtkWidget *spinner;
//     GtkWidget *button;
// } AppWidgets;
//
// static void display_output(AppWidgets *widgets, const gchar *output) {
//     gtk_text_buffer_set_text(widgets->output_buffer, output, -1);
//     gtk_spinner_stop(GTK_SPINNER(widgets->spinner));
//     gtk_widget_set_sensitive(widgets->button, TRUE);
// }
//
// static void on_command_finished(GObject *source_object, GAsyncResult *res, gpointer user_data) {
//     GtkTextView *text_view = GTK_TEXT_VIEW(user_data);
//     GSubprocess *subprocess = G_SUBPROCESS(source_object);
//     g_autoptr(GError) error = NULL;
//     gchar *stdout_str = NULL;
//
//     g_subprocess_communicate_utf8_finish(subprocess, res, &stdout_str, NULL, &error);
//     if (error) {
//         g_printerr("Failed to run command: %s\n", error->message);
//         return;
//     }
//
//     GtkTextBuffer *buffer = gtk_text_view_get_buffer(text_view);
//     gtk_text_buffer_set_text(buffer, stdout_str, -1);
//     g_free(stdout_str);
// }
//
// static void on_button_clicked(GtkButton *button, gpointer user_data) {
//     AppWidgets *widgets = user_data;
//     const gchar *username = gtk_editable_get_text(GTK_EDITABLE(widgets->entry));
//
//     if (!username || *username == '\0') {
//         display_output(widgets, "Please enter a username.");
//         return;
//     }
//
//     gtk_text_buffer_set_text(widgets->output_buffer, "Fetching data...", -1);
//     gtk_spinner_start(GTK_SPINNER(widgets->spinner));
//     gtk_widget_set_sensitive(widgets->button, FALSE);
//
//     gchar *argv[] = {
//         "/home/izelnakri/Github/c-projects/c/build/github_user_fetcher",
//         (gchar *)username,
//         NULL
//     };
//
//     GSubprocess *subproc = g_subprocess_new(
//         G_SUBPROCESS_FLAGS_STDOUT_PIPE | G_SUBPROCESS_FLAGS_STDERR_PIPE,
//         NULL, argv[0], argv[1], NULL
//     );
//
//     if (!subproc) {
//         display_output(widgets, "Failed to start process.");
//         return;
//     }
//
//     g_subprocess_communicate_utf8_async(
//         subproc, NULL, NULL, on_command_finished, widgets
//     );
// }

// static void activate(GtkApplication *app, gpointer user_data) {
//     GtkWidget *window = gtk_application_window_new(app);
//     gtk_window_set_title(GTK_WINDOW(window), "GitHub User Fetcher");
//     gtk_window_set_default_size(GTK_WINDOW(window), 600, 400);
//
//     GtkCssProvider *provider = gtk_css_provider_new();
//     gtk_css_provider_load_from_path(provider, "style.css");
//     gtk_style_context_add_provider_for_display(
//         gdk_display_get_default(),
//         GTK_STYLE_PROVIDER(provider),
//         GTK_STYLE_PROVIDER_PRIORITY_APPLICATION
//     );
//
//     GtkWidget *box = gtk_box_new(GTK_ORIENTATION_VERTICAL, 10);
// //     // styling:
// //     gtk_widget_set_margin_top(box, 20);
// //     gtk_widget_set_margin_bottom(box, 20);
// //     gtk_widget_set_margin_start(box, 20);
// //     gtk_widget_set_margin_end(box, 20);
// //     // styling end
//     gtk_window_set_child(GTK_WINDOW(window), box);
//
//     AppWidgets *widgets = g_new0(AppWidgets, 1);
//
//     widgets->entry = GTK_ENTRY(gtk_entry_new());
//     gtk_entry_set_placeholder_text(widgets->entry, "Enter GitHub username");
//     gtk_widget_add_css_class(GTK_WIDGET(widgets->entry), "username-entry");
//     gtk_box_append(GTK_BOX(box), GTK_WIDGET(widgets->entry));
//
//     widgets->button = gtk_button_new_with_label("Fetch");
//     gtk_widget_add_css_class(widgets->button, "fetch-button");
//     gtk_box_append(GTK_BOX(box), widgets->button);
//
//     widgets->spinner = gtk_spinner_new();
//     gtk_box_append(GTK_BOX(box), widgets->spinner);
//
//     GtkWidget *scrolled = gtk_scrolled_window_new();
//     gtk_widget_set_vexpand(scrolled, TRUE);
//     gtk_box_append(GTK_BOX(box), scrolled);
//
//     GtkWidget *output_view = gtk_text_view_new();
//     gtk_text_view_set_editable(GTK_TEXT_VIEW(output_view), FALSE);
//     gtk_text_view_set_wrap_mode(GTK_TEXT_VIEW(output_view), GTK_WRAP_WORD_CHAR);
//     gtk_widget_add_css_class(output_view, "output-view");
//
//     gtk_scrolled_window_set_child(GTK_SCROLLED_WINDOW(scrolled), output_view);
//
//     widgets->output_buffer = gtk_text_view_get_buffer(GTK_TEXT_VIEW(output_view));
//
//     g_signal_connect(widgets->button, "clicked", G_CALLBACK(on_button_clicked), widgets);
//     g_signal_connect(window, "close-request", G_CALLBACK(gtk_window_destroy), NULL);
//
//     gtk_window_present(GTK_WINDOW(window));
// }
//
// int main(int argc, char *argv[]) {
//     GtkApplication *app = gtk_application_new("com.example.githubfetcher", G_APPLICATION_FLAGS_NONE);
//     g_signal_connect(app, "activate", G_CALLBACK(activate), NULL);
//     int status = g_application_run(G_APPLICATION(app), argc, argv);
//     g_object_unref(app);
//     return status;
// }
