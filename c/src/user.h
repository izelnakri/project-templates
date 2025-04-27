#ifndef USER_H
#define USER_H

typedef struct {
  char login[256];
  char name[256];
  char company[256];
  char location[256];
} User;

#ifdef __cplusplus
extern "C" {
#endif
User fetch_github_user(const char *username);

void print_github_user(const User *user);
#ifdef __cplusplus
}
#endif

#endif
