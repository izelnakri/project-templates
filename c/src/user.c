#include "user.h"
#include <curl/curl.h>
#include <jansson.h>
#include <stdio.h>
#include <string.h>

static size_t write_callback(void *contents, size_t size, size_t nmemb,
                             void *userp) {
  size_t total_size = size * nmemb;
  strncat((char *)userp, contents, total_size);
  return total_size;
}

User fetch_github_user(const char *username) {
  User user = {0};
  char url[256], buffer[4096] = {0};

  snprintf(url, sizeof(url), "https://api.github.com/users/%s", username);

  CURL *curl = curl_easy_init();
  if (!curl) {
    fprintf(stderr, "curl init failed\n");
    return user;
  }

  curl_easy_setopt(curl, CURLOPT_URL, url);
  curl_easy_setopt(curl, CURLOPT_USERAGENT, "github_user_fetcher/1.0");
  curl_easy_setopt(curl, CURLOPT_WRITEFUNCTION, write_callback);
  curl_easy_setopt(curl, CURLOPT_WRITEDATA, buffer);

  CURLcode res = curl_easy_perform(curl);
  if (res != CURLE_OK) {
    fprintf(stderr, "CURL error: %s\n", curl_easy_strerror(res));
    curl_easy_cleanup(curl);
    return user;
  }
  curl_easy_cleanup(curl);

  json_error_t error;
  json_t *root = json_loads(buffer, 0, &error);
  if (!root) {
    fprintf(stderr, "JSON parse error: %s\n", error.text);
    return user;
  }

  json_t *login = json_object_get(root, "login");
  json_t *name = json_object_get(root, "name");
  json_t *company = json_object_get(root, "company");
  json_t *location = json_object_get(root, "location");

  if (json_is_string(login))
    strncpy(user.login, json_string_value(login), sizeof(user.login));
  if (json_is_string(name))
    strncpy(user.name, json_string_value(name), sizeof(user.name));
  if (json_is_string(company))
    strncpy(user.company, json_string_value(company), sizeof(user.company));
  if (json_is_string(location))
    strncpy(user.location, json_string_value(location), sizeof(user.location));

  json_decref(root);
  return user;
}

void print_github_user(const User *user) {
  printf("GitHub User:\n");
  printf("  Login:   %s\n", user->login);
  printf("  Name:    %s\n", user->name);
  printf("  Company: %s\n", user->company);
  printf("  Location:%s\n", user->location);
}
