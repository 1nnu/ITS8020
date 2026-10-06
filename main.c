#include <errno.h>
#include <pwd.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#include "auth.h"

static void usage(const char *prog) {
    fprintf(stderr, "Usage: %s <command> [args...]\n", prog);
}

int main(int argc, char *argv[]) {
    if (argc < 2) {
        usage(argv[0]);
        return EXIT_FAILURE;
    }

    struct passwd *pw = getpwuid(getuid());

    if (!authenticate(pw ? pw->pw_name : "")) {
        fprintf(stderr, "%s: Sorry, try again.\n", argv[0]);
        return EXIT_FAILURE;
    }

    execvp(argv[1], &argv[1]);

    fprintf(stderr, "%s: %s: %s\n", argv[0], argv[1], strerror(errno));
    return EXIT_FAILURE;
}
