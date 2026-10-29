#include <getopt.h>
#include <pwd.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

#include "auth.h"
#include "run.h"

/* Random custom error codes */
#define ERR_USAGE 2
#define ERR_AUTH  5

/* LLM suggested refactoring for the usage command */
static void usage(const char *prog) {
    fprintf(stderr,
            "Usage: %s [-u user] [--help] command [args...]\n"
            "  -u user      run command as user (default: root)\n"
            "  --help       show this help and exit\n",
            prog);
}

int main(int argc, char *argv[]) {
    const char *target_user = "root";

    /* Taken from https://stackoverflow.com/questions/7489093/getopt-long-proper-way-to-use-it */
    /* Maps --help to -h */
    static struct option long_options[] = {
        {"help", no_argument, NULL, 'h'},
        {0, 0, 0, 0}
    };

    /* Parsing of cli args */
    int opt;
    while ((opt = getopt_long(argc, argv, "+u:h", long_options, NULL)) != -1) {
        switch (opt) {
        case 'u':
            target_user = optarg;
            break;
        case 'h':
            usage(argv[0]);
            return EXIT_SUCCESS;
        default:
            usage(argv[0]);
            return ERR_USAGE;
        }
    }

    /* Check that at least 1 arg was provided for sudo to run */
    if (optind >= argc) {
        usage(argv[0]);
        return ERR_USAGE;
    }

    /* Get password for command runner */
    struct passwd *invoker = getpwuid(getuid());

    /* Authenticate */
    if (!authenticate(invoker->pw_name)) {
        fprintf(stderr, "%s: Sorry, try again.\n", argv[0]);
        return ERR_AUTH;
    }

    /* Get password for target user */
    struct passwd *target = getpwnam(target_user);
    setuid(target->pw_uid);

    /* Set safe paths */
    setenv("PATH", "/usr/bin:/bin", 1);

    /* Run target command */
    return run_command(&argv[optind]);
}
