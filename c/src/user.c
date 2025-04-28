#include "user.h"
#include <curl/curl.h>
#include <jansson.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

/**
 * @brief Buffer structure to hold incoming HTTP response data.
 */
typedef struct {
  char *data;
  size_t size;
  size_t capacity; // Added capacity field to track buffer size
} Buffer;

/**
 * @brief Initialize a Buffer structure.
 *
 * @param buf Pointer to Buffer to initialize.
 */
void buffer_init(Buffer *buf) {
  buf->data = malloc(1024); // Initial allocation
  buf->size = 0;
  buf->capacity = 1024;
  if (!buf->data) {
    (void)fprintf(stderr, "Memory allocation failed\n");
  }
}

/**
 * @brief Free the memory held by a Buffer.
 *
 * @param buf Pointer to Buffer to clean up.
 */
void buffer_cleanup(Buffer *buf) {
  if (buf && buf->data) {
    free(buf->data);
    buf->data = NULL;
    buf->size = 0;
    buf->capacity = 0;
  }
}

/**
 * @brief Ensure that a Buffer has enough capacity for new data.
 *
 * @param buf Pointer to Buffer.
 * @param required_size Size needed.
 * @return 1 if successful, 0 on failure.
 */
int buffer_ensure_capacity(Buffer *buf, size_t required_size) {
  if (buf->capacity >= required_size) {
    return 1; // Enough space
  }

  // Double the capacity to ensure enough room
  size_t new_capacity = buf->capacity * 2;
  while (new_capacity < required_size) {
    new_capacity *= 2;

    // Protect against integer overflow
    if (new_capacity < buf->capacity) {
      (void)fprintf(stderr, "Buffer capacity overflow\n");
      return 0;
    }
  }

  char *new_data = realloc(buf->data, new_capacity);
  if (!new_data) {
    (void)fprintf(stderr, "Memory allocation failed for buffer\n");
    return 0;
  }

  buf->data = new_data;
  buf->capacity = new_capacity;
  return 1;
}

/**
 * @brief Callback for libcurl to write response data into a Buffer.
 *
 * @param contents Pointer to incoming data.
 * @param size Size of a single item.
 * @param nmemb Number of items.
 * @param userp Pointer to Buffer structure.
 * @return Number of bytes handled.
 */
static size_t write_callback(const void *contents, size_t size, size_t nmemb,
                             void *userp) {
  size_t total_size = size * nmemb;
  Buffer *buf = (Buffer *)userp;

  // Check for size_t overflow
  if (total_size / size != nmemb) {
    (void)fprintf(stderr, "Integer overflow in size calculation\n");
    return 0;
  }

  // Check for overflow when adding to buffer size
  if (total_size > SIZE_MAX - buf->size - 1) {
    (void)fprintf(stderr, "Buffer size would overflow\n");
    return 0;
  }

  // Ensure there's enough space for the new data
  if (!buffer_ensure_capacity(buf, buf->size + total_size + 1)) {
    return 0;
  }

  // Now we can safely copy the data
  if (total_size > 0) {
    memmove(buf->data + buf->size, contents,
            total_size); // Using memmove instead of memcpy
  }
  buf->size += total_size;
  buf->data[buf->size] = '\0'; // Null-terminate

  return total_size;
}

/**
 * @brief Fetch a GitHub user's information via the GitHub API.
 *
 * @param username GitHub username.
 * @return User structure containing fetched information.
 *
 * The returned User must be freed using user_free().
 */
User fetch_github_user(const char *username) {
  User user = {0};
  Buffer buffer = {0};

  // Check username for NULL or empty
  if (!username || *username == '\0') {
    (void)fprintf(stderr, "Invalid username\n");
    return user;
  }

  buffer_init(&buffer); // Initialize buffer after username check to avoid leaks

  // Dynamically allocate URL buffer with enough space for the URL
  size_t username_len = strnlen(username, 100); // Limit username length
  size_t url_size =
      username_len + 50; // Account for the API URL length and username

  char *url = malloc(url_size);
  if (!url) {
    (void)fprintf(stderr, "Memory allocation failed for URL\n");
    buffer_cleanup(&buffer);
    return user;
  }

  int ret =
      snprintf(url, url_size, "https://api.github.com/users/%s", username);
  if (ret < 0 || (size_t)ret >= url_size) {
    (void)fprintf(stderr, "Error: URL was truncated or snprintf failed\n");
    free(url);
    buffer_cleanup(&buffer);
    return user;
  }

  CURL *curl = curl_easy_init();
  if (!curl) {
    (void)fprintf(stderr, "curl init failed\n");
    free(url);
    buffer_cleanup(&buffer);
    return user;
  }

  curl_easy_setopt(curl, CURLOPT_URL, url);
  curl_easy_setopt(curl, CURLOPT_USERAGENT, "github_user_fetcher/1.0");
  curl_easy_setopt(curl, CURLOPT_WRITEFUNCTION, write_callback);
  curl_easy_setopt(curl, CURLOPT_WRITEDATA, &buffer);

  CURLcode res = curl_easy_perform(curl);
  curl_easy_cleanup(curl);

  free(url); // Free dynamically allocated URL buffer

  if (res != CURLE_OK) {
    (void)fprintf(stderr, "CURL error: %s\n", curl_easy_strerror(res));
    buffer_cleanup(&buffer);
    return user;
  }

  json_error_t error;
  json_t *root = json_loads(buffer.data, 0, &error);

  // Save buffer data before cleanup
  char *buffer_data = buffer.data;

  if (!root) {
    (void)fprintf(stderr, "JSON parse error: %s\n", error.text);
    free(buffer_data);
    return user;
  }

  free(buffer_data); // Free buffer data after successful JSON parsing

  json_t *login = json_object_get(root, "login");
  json_t *name = json_object_get(root, "name");
  json_t *company = json_object_get(root, "company");
  json_t *location = json_object_get(root, "location");

  // Dynamically allocate memory for the user fields
  if (json_is_string(login)) {
    user.login = strdup(
        json_string_value(login)); // Using strdup for dynamic memory allocation
  }
  if (json_is_string(name)) {
    user.name = strdup(json_string_value(name));
  }
  if (json_is_string(company)) {
    user.company = strdup(json_string_value(company));
  }
  if (json_is_string(location)) {
    user.location = strdup(json_string_value(location));
  }

  json_decref(root);
  return user;
}

/**
 * @brief Print a User's information to the console.
 *
 * @param user Pointer to the User structure.
 */
void print_github_user(const User *user) {
  if (!user) {
    (void)fprintf(stderr, "NULL user pointer\n");
    return;
  }

  printf("GitHub User:\n");
  printf("  Login:    %s\n", user->login ? user->login : "N/A");
  printf("  Name:     %s\n", user->name ? user->name : "N/A");
  printf("  Company:  %s\n", user->company ? user->company : "N/A");
  printf("  Location: %s\n", user->location ? user->location : "N/A");
}

/**
 * @brief Free memory used by a User structure.
 *
 * @param user Pointer to User.
 *
 * After freeing, all pointers are set to NULL.
 */
void user_free(User *user) {
  if (!user) {
    return;
  }

  free(user->login);
  free(user->name);
  free(user->company);
  free(user->location);

  // Zero out pointers after freeing
  user->login = NULL;
  user->name = NULL;
  user->company = NULL;
  user->location = NULL;
}
