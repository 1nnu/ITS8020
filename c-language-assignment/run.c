#include "run.h"

#include <stdio.h>
#include <stdlib.h>
#include <sys/wait.h>
#include <unistd.h>

/* https://stackoverflow.com/questions/16215976/fork-function-will-never-return-0 for use of forking logic */
int run_command(char *const argv[]) {
    pid_t pid = fork();

    /* If child run the command */
    if (pid == 0) {
        execvp(argv[0], argv);
        _exit(127);
    }

    /* Get status of pid */
    int status;
    waitpid(pid, &status, 0);

    /* Return 0 if child exited successfully */
    return WEXITSTATUS(status);
}
