#include "user.h"
#include <stdio.h>
#include <string.h>

int main(int argc, char **argv) {
    const char *username = "izelnakri";

    for (int i = 1; i < argc; i++) {
        if (strcmp(argv[i], "--user") == 0 && i + 1 < argc) {
            username = argv[i + 1];
            i++;
        } else if (strncmp(argv[i], "--user=", 7) == 0) {
            username = argv[i] + 7;
        }
    }

    User user = fetch_github_user(username);
    print_github_user(&user);

    return 0;
}
