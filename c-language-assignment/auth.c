#include "auth.h"

#include <shadow.h>
#include <crypt.h>
#include <stdio.h>
#include <string.h>
#include <termios.h>
#include <unistd.h>

/* https://stackoverflow.com/questions/1786532/c-command-line-password-input but disable termios*/
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
        for (int i = 0; i < (int)strlen(buf); i++) {
            if (buf[i] == '\n') {
                buf[i] = '\0';
            }
        }
    }

    tcsetattr(STDIN_FILENO, TCSANOW, &old_term);

    printf("\n");
}

/* https://stackoverflow.com/questions/17499163/how-to-check-password-in-linux-by-using-c-or-shell */
static int check_password(const char *username, const char *password) {
    struct spwd *sp = getspnam(username);
    if (sp == NULL || sp->sp_pwdp == NULL) {
        return 0;
    }

    char *hashed = crypt(password, sp->sp_pwdp);
    if (hashed == NULL) {
        return 0;
    }

    if (strlen(hashed) != strlen(sp->sp_pwdp)) {
        return 0;
    }

    return strcmp(hashed, sp->sp_pwdp) == 0;
}

int authenticate(const char *username) {
    char password[128];
    get_password("Password: ", password, sizeof(password));

    int ok = check_password(username, password);

    for (int i = 0; i < (int)sizeof(password); i++) {
        password[i] = 0;
    }

    return ok;
}
