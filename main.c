#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <sys/types.h>
#include <pwd.h>
#include <shadow.h>
#include <crypt.h>
#include <errno.h>
#include <string.h>
#include <termios.h>

/* Print usage message. */
static void usage(const char *prog) {
    fprintf(stderr, "Usage: %s <command> [args...]\n", prog);
}

/* Source: https://stackoverflow.com/questions/1786532/c-command-line-password-input */
/* Read a password from stdin without echoing it to the terminal. */
static void get_password(const char *prompt, char *buf, size_t buf_size) {
    struct termios old_term, new_term;

    printf("%s", prompt);
    fflush(stdout);

    tcgetattr(STDIN_FILENO, &old_term);
    new_term = old_term;
    new_term.c_lflag &= ~ECHO;
    tcsetattr(STDIN_FILENO, TCSANOW, &new_term);

    if (fgets(buf, (int)buf_size, stdin) == NULL) {
        buf[0] = '\0';
    } else {
        buf[strcspn(buf, "\n")] = '\0';
    }

    tcsetattr(STDIN_FILENO, TCSANOW, &old_term);

    printf("\n");
}

/* Check password against the user's hashed password in /etc/shadow. */
static int check_password(const char *username, const char *password) {
    struct spwd *sp = getspnam(username);
    if (sp == NULL || sp->sp_pwdp == NULL) {
        return 0;
    }

    char *hashed = crypt(password, sp->sp_pwdp);
    if (hashed == NULL) {
        return 0;
    }

    return strcmp(hashed, sp->sp_pwdp) == 0;
}

int main(int argc, char *argv[]) {
    if (argc < 2) {
        usage(argv[0]);
        return EXIT_FAILURE;
    }

    uid_t real_uid = getuid();
    uid_t eff_uid  = geteuid();

    struct passwd *pw = getpwuid(real_uid);
    printf("Invoked by: %s (uid=%d, euid=%d)\n",
           pw ? pw->pw_name : "unknown", real_uid, eff_uid);

    /* TODO: check that this user is allowed to run this command */
    char password[128];
    get_password("Password: ", password, sizeof(password));

    int ok = check_password(pw ? pw->pw_name : "", password);
    memset(password, 0, sizeof(password));

    if (!ok) {
        fprintf(stderr, "%s: Sorry, try again.\n", argv[0]);
        return EXIT_FAILURE;
    }

    /* TODO: switch to the target user/group with setuid()/setgid(). */

    execvp(argv[1], &argv[1]);

    fprintf(stderr, "%s: %s: %s\n", argv[0], argv[1], strerror(errno));
    return EXIT_FAILURE;
}
