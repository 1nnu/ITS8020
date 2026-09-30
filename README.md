# minimal-sudo

A minimal clone of `sudo` for ITS8020. It asks for the invoking user's
password, checks it against `/etc/shadow`, and if it matches, runs the
given command.

```
minimal-sudo <command> [args...]
```

## Build

```
make
```

This produces the `minimal-sudo` binary.

## Make it work

Password verification reads `/etc/shadow`, which normal users can't
read. For the check to succeed, the binary needs to run as root, so
install it setuid-root:

```
sudo chown root:root minimal-sudo
sudo chmod u+s minimal-sudo
```

Then run it as your normal user:

```
./minimal-sudo echo hello
```

You'll be prompted for your own login password; enter it correctly and
the command runs.

## Clean

```
make clean
```
