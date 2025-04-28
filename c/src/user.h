#ifndef USER_H
#define USER_H

#include <stddef.h> // for size_t

/**
 * @brief Structure representing a GitHub user.
 */
typedef struct {
  char *login;    ///< GitHub username
  char *name;     ///< Full name
  char *company;  ///< Company name
  char *location; ///< User's location
} User;

#ifdef __cplusplus
extern "C" {
#endif

/**
 * @brief Fetch GitHub user information for a given username.
 *
 * @param username GitHub username to fetch.
 * @return User structure populated with user information.
 *
 * The returned User must be freed with user_free().
 */
User fetch_github_user(const char *username);

/**
 * @brief Print GitHub user information to standard output.
 *
 * @param user Pointer to a User structure.
 */
void print_github_user(const User *user);

/**
 * @brief Free dynamically allocated memory in a User structure.
 *
 * @param user Pointer to a User structure to free.
 *
 * After calling, the fields inside User will be set to NULL.
 */
void user_free(User *user); // New function to free dynamically allocated memory

#ifdef __cplusplus
}
#endif

#endif
