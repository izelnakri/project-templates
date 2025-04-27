#ifndef USER_H
#define USER_H

#include <stddef.h> // for size_t

typedef struct {
  char *login;
  char *name;
  char *company;
  char *location;
} User;

#ifdef __cplusplus
extern "C" {
#endif

User fetch_github_user(const char *username);
void print_github_user(const User *user);
void user_free(User *user); // New function to free dynamically allocated memory

#ifdef __cplusplus
}
#endif

#endif
