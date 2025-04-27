#include "../src/user.h"
#include <criterion/criterion.h>

Test(user, fetch_existing_user) {
  User user = fetch_github_user("octocat");
  cr_assert_str_eq(user.login, "octocat");
}
