nix-simple-deploy
=================
![](https://github.com/misuzu/nix-simple-deploy/workflows/Continuous%20integration/badge.svg)

## About

Deploy a NixOS system configuration with `nix-simple-deploy system ...` to a remote
machine and switch the machine to that system configuration. You can also deploy
a nix store path with `nix-simple-deploy path ...` to a remote machine.

This is a Rust rewrite of unmaintained [nix-deploy](https://github.com/awakesecurity/nix-deploy).

## Usage

To get started generate signing key first:
```bash
$ nix-store --generate-binary-cache-key cache.example.com-1 signing-key.sec signing-key.pub
```

Then add contents of ```signing-key.pub``` to remote's ```configuration.nix``` and run ```nixos-rebuild switch```:
```nix
{
  nix.binaryCachePublicKeys = [ "cache.example.com-1:<STRING FROM signing-key.pub>" ];
}
```

Now you are ready to deploy stuff!

To simply copy some store path use `nix-simple-deploy path`:
```bash
$ nix-simple-deploy path \
  --use-remote-sudo \
  --use-substitutes \
  --signing-key signing-key.sec \
  --target-host user@remote-server \
  $(type -p firefox)
```

To deploy whole system use `nix-simple-deploy system`:
```bash
# copy hardware configuration from remote host to hardware-configuration.nix
$ ssh user@remote-server nixos-generate-config --show-hardware-config > ./hardware-configuration.nix
# build system from configuration.nix
$ nix-build '<nixpkgs/nixos>' -Q -A system -I nixos-config=./configuration.nix
# deploy system to remote host
$ nix-simple-deploy system \
  --use-remote-sudo \
  --use-substitutes \
  --signing-key signing-key.sec \
  --target-host user@remote-server \
  $(readlink -f ./result) \
  switch
```

## Install

Just add it to `environment.systemPackages` (nixpkgs-unstable):
```nix
{
  environment.systemPackages = [
    pkgs.nix-simple-deploy
  ];
}
```

To run ```nix-simple-deploy``` from git tree run:
```bash
$ nix-shell -p cargo -p nix-serve
$ cargo run -- --help
```

You can also build `nix-simple-deploy` directly from provided `default.nix` expression from this repo. Just setup `rev` value and appropriate `sha256`:

```nix
{
  nix-simple-deploy = pkgs.callPackage (pkgs.fetchFromGitHub {
    rev = "...";
    owner = "misuzu";
    repo = "nix-simple-deploy";
    sha256 = "...";
  }) {};
}
```
Then you can add it to your `shell.nix` `buildInputs` or system wide into `environment.systemPackages`:

```nix
{
  environment.systemPackages = [
    nix-simple-deploy
  ];
}
```

## Help output

```bash
$ nix-simple-deploy --help
Deploy software or an entire NixOS system configuration to another NixOS system

USAGE:
    nix-simple-deploy <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    help      Prints this message or the help of the given subcommand(s)
    path      Deploy a path to the NixOS target host
    system    Deploy a system to the NixOS target host
```

```bash
$ nix-simple-deploy path --help
Deploy a path to the NixOS target host

USAGE:
    nix-simple-deploy path [FLAGS] <PATH> --nix-serve-port <nix-serve-port> --signing-key </path/to/signing-key> --target-host <USER@HOST>

FLAGS:
    -h, --help               Prints help information
    -p, --profile-path       Profile path
        --use-remote-sudo    When set, nix-simple-deploy prefixes remote commands that run on the --target-host systems
                             with sudo. Setting this option allows deploying using remote non-root user
    -s, --use-substitutes    Attempt to download missing paths on the target machine using Nix’s substitute mechanism.
                             Any paths that cannot be substituted on the target are still copied normally from the
                             source

OPTIONS:
    -n, --nix-serve-port <nix-serve-port>       Port used for nix-serve, use this option if you have other services that
                                                use port 9999 on local or remote machine [default: 9999]
    -k, --signing-key </path/to/signing-key>    File containing the secret signing key
    -t, --target-host <USER@HOST>               Specifies the NixOS target host

ARGS:
    <PATH>    Nix store path
```

```bash
$ nix-simple-deploy system --help
Deploy a system to the NixOS target host

USAGE:
    nix-simple-deploy system [FLAGS] <PATH> <ACTION> --nix-serve-port <nix-serve-port> --profile-path <profile-path> --signing-key </path/to/signing-key> --target-host <USER@HOST>

FLAGS:
    -h, --help               Prints help information
        --use-remote-sudo    When set, nix-simple-deploy prefixes remote commands that run on the --target-host systems
                             with sudo. Setting this option allows deploying using remote non-root user
    -s, --use-substitutes    Attempt to download missing paths on the target machine using Nix’s substitute mechanism.
                             Any paths that cannot be substituted on the target are still copied normally from the
                             source

OPTIONS:
    -n, --nix-serve-port <nix-serve-port>       Port used for nix-serve, use this option if you have other services that
                                                use port 9999 on local or remote machine [default: 9999]
    -p, --profile-path <profile-path>           Profile path [default: /nix/var/nix/profiles/system]
    -k, --signing-key </path/to/signing-key>    File containing the secret signing key
    -t, --target-host <USER@HOST>               Specifies the NixOS target host

ARGS:
    <PATH>      Nix store path
    <ACTION>    Desired operation [possible values: switch, boot, test, dry-activate, reboot]
```
