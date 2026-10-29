# minimal-sudo

A minimal clone of `sudo` for ITS8020, assignment 01.

It checks the invoking user's own password against `/etc/shadow`, and
if it's correct, runs the given command as another user (root by
default).

## Usage

```
minimal-sudo [-u user] [--help] command [args...]
```

- `-u user` - run the command as `user` instead of root.
- `--help` - print usage and exit.

## Build

```
make
```

This compiles `main.c`, `auth.c` and `run.c` into the `minimal-sudo`
binary.

## Make it work

Checking the password requires reading `/etc/shadow`, and actually
switching user requires `setuid(2)`. Both need root privileges, so the
binary has to be installed setuid-root:

```
make install
```

(runs `sudo chown root:root` + `sudo chmod u+s` on the binary; you'll
need to do this again after every `make clean && make`, since
rebuilding resets ownership and the setuid bit.)

Then run it as your normal user:

```
./minimal-sudo echo hello
./minimal-sudo -u www-data whoami
```

You'll be prompted for your own login password; enter it correctly and
the command runs as the target user. The program exits with the
command's own exit code.

## Clean

```
make clean
```

## Tested on

Ubuntu 26.04 (Linux), gcc.

## AI usgae

AI was used to generate README.md and maintain the Makefile

## License

GPL-3.0-or-later, see `LICENSE`.
