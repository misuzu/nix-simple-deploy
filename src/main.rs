use clap::*;
use subprocess::{Exec, NullFile, Result};

use std::fs;
use std::process::exit;
use std::time;

fn get_ssh_tool(
    target_host: &str,
    extra_ssh_options: Option<&str>,
    nix_serve_port: u16,
    use_remote_sudo: bool,
) -> String {
    let cmd = format!(
        "ssh {} -R {}:127.0.0.1:{} {}",
        extra_ssh_options.unwrap_or(""),
        nix_serve_port,
        nix_serve_port,
        target_host
    );
    if use_remote_sudo {
        format!("{} sudo", cmd)
    } else {
        cmd
    }
}

fn deploy_path(
    ssh_tool: &str,
    nix_serve_port: u16,
    use_substitutes: bool,
    path: &str,
    signing_key: Option<&str>,
    store: Option<&str>,
    profile_path: Option<&str>,
) -> Result<()> {
    let mut cmd = Exec::cmd("nix-serve")
        .arg("-p")
        .arg(nix_serve_port.to_string())
        .stdout(NullFile)
        .stderr(NullFile);
    if let Some(key_path) = signing_key {
        cmd = cmd.env("NIX_SECRET_KEY_FILE", key_path);
    };
    match cmd.popen() {
        Ok(ref mut nix_serve) => {
            if let Some(exit_status) = nix_serve.wait_timeout(time::Duration::from_secs(1))? {
                println!("nix-serve exited too early: {:?}", exit_status);
                exit(1);
            };

            let path = fs::read_link(path)
                .unwrap_or_else(|_| path.into())
                .as_path()
                .display()
                .to_string();

            let cmd = if let Some(profile_path) = profile_path {
                format!(
                    "{} nix-env --option {}substituters http://127.0.0.1:{} {} {} -p {} --set {}",
                    ssh_tool,
                    (if use_substitutes { "extra-" } else { "" }),
                    nix_serve_port,
                    signing_key.map_or("--option require-sigs false", |_| ""),
                    store.map_or("".to_string(), |s| format!("--store {}", s)),
                    profile_path,
                    path
                )
            } else {
                format!(
                    "{} nix build --option {}substituters http://127.0.0.1:{} {} {} --print-missing -v --no-link {}",
                    ssh_tool,
                    (if use_substitutes { "extra-" } else { "" }),
                    nix_serve_port,
                    signing_key.map_or("--option require-sigs false", |_| ""),
                    store.map_or("".to_string(), |s| format!("--store {}", s)),
                    path
                )
            };

            let exit_status = Exec::shell(cmd).join()?;

            nix_serve.terminate()?;

            if !exit_status.success() {
                exit(1);
            }

            Ok(())
        }
        Err(e) => {
            println!("Error while starting nix-serve:");
            Err(e)
        }
    }
}

fn deploy_system(
    ssh_tool: &str,
    nix_serve_port: u16,
    use_substitutes: bool,
    path: &str,
    signing_key: Option<&str>,
    action: &str,
    profile_path: &str,
) -> Result<()> {
    let remote_action = if action == "reboot" { "boot" } else { action };

    let profile_path = match remote_action {
        "switch" | "boot" => Some(profile_path),
        _ => None,
    };

    deploy_path(
        ssh_tool,
        nix_serve_port,
        use_substitutes,
        path,
        signing_key,
        None,
        profile_path.as_deref(),
    )?;

    let cmd = format!(
        "{} {}/bin/switch-to-configuration {}",
        ssh_tool,
        profile_path.as_deref().unwrap_or(path),
        remote_action
    );

    let exit_status = Exec::shell(cmd).join()?;

    if !exit_status.success() {
        exit(1);
    }

    if action == "reboot" {
        let mut p = Exec::shell(format!("{} reboot", ssh_tool))
            .detached()
            .popen()?;
        let _ = p.wait_timeout(time::Duration::from_secs(10));
    }

    Ok(())
}

fn main() {
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .about(crate_description!())
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .setting(AppSettings::VersionlessSubcommands)
        .subcommand(
            SubCommand::with_name("path")
                .about("Deploy a path to the NixOS target host")
                .arg(
                    Arg::with_name("target-host")
                        .short("t")
                        .long("target-host")
                        .help("Specifies the NixOS target host")
                        .value_name("USER@HOST")
                        .required(true),
                )
                .arg(
                    Arg::with_name("extra-ssh-options")
                        .long("extra-ssh-options")
                        .help("Extra options for ssh binary")
                        .value_name("ssh-options"),
                )
                .arg(
                    Arg::with_name("nix-serve-port")
                        .short("n")
                        .long("nix-serve-port")
                        .help(
                            "Port used for nix-serve, use this option \
                             if you have other services that use port 9999 \
                             on local or remote machine",
                        )
                        .value_name("port")
                        .default_value("9999"),
                )
                .arg(
                    Arg::with_name("signing-key")
                        .short("k")
                        .long("signing-key")
                        .help("File containing the secret signing key")
                        .value_name("/path/to/signing-key"),
                )
                .arg(
                    Arg::with_name("store")
                        .long("store")
                        .help("Use different nix store root")
                        .value_name("/mnt"),
                )
                .arg(
                    Arg::with_name("use-substitutes")
                        .short("s")
                        .long("use-substitutes")
                        .help(
                            "Attempt to download missing paths on the target \
                             machine using Nix’s substitute mechanism. \
                             Any paths that cannot be substituted on the \
                             target are still copied normally from the source",
                        ),
                )
                .arg(
                    Arg::with_name("use-remote-sudo")
                        .long("use-remote-sudo")
                        .help(
                            "When set, nix-simple-deploy prefixes remote commands \
                             that run on the --target-host systems with sudo. \
                             Setting this option allows deploying using remote non-root user",
                        ),
                )
                .arg(
                    Arg::with_name("profile-path")
                        .short("p")
                        .long("profile-path")
                        .help("Profile path")
                        .value_name("/path/to/nix/profile"),
                )
                .arg(Arg::with_name("PATH").help("Nix store path").required(true)),
        )
        .subcommand(
            SubCommand::with_name("system")
                .about("Deploy a system to the NixOS target host")
                .arg(
                    Arg::with_name("target-host")
                        .short("t")
                        .long("target-host")
                        .help("Specifies the NixOS target host")
                        .value_name("USER@HOST")
                        .required(true),
                )
                .arg(
                    Arg::with_name("extra-ssh-options")
                        .long("extra-ssh-options")
                        .help("Extra options for ssh binary")
                        .value_name("ssh-options"),
                )
                .arg(
                    Arg::with_name("nix-serve-port")
                        .short("n")
                        .long("nix-serve-port")
                        .help(
                            "Port used for nix-serve, use this option \
                             if you have other services that use port 9999 \
                             on local or remote machine",
                        )
                        .value_name("port")
                        .default_value("9999"),
                )
                .arg(
                    Arg::with_name("signing-key")
                        .short("k")
                        .long("signing-key")
                        .help("File containing the secret signing key")
                        .value_name("/path/to/signing-key"),
                )
                .arg(
                    Arg::with_name("use-substitutes")
                        .short("s")
                        .long("use-substitutes")
                        .help(
                            "Attempt to download missing paths on the target \
                             machine using Nix’s substitute mechanism. \
                             Any paths that cannot be substituted on the \
                             target are still copied normally from the source",
                        ),
                )
                .arg(
                    Arg::with_name("use-remote-sudo")
                        .long("use-remote-sudo")
                        .help(
                            "When set, nix-simple-deploy prefixes remote commands \
                             that run on the --target-host systems with sudo. \
                             Setting this option allows deploying using remote non-root user",
                        ),
                )
                .arg(
                    Arg::with_name("profile-path")
                        .short("p")
                        .long("profile-path")
                        .help("Profile path")
                        .value_name("/path/to/nix/profile")
                        .default_value("/nix/var/nix/profiles/system"),
                )
                .arg(Arg::with_name("PATH").help("Nix store path").required(true))
                .arg(
                    Arg::with_name("ACTION")
                        .help("Desired operation")
                        .possible_values(&["switch", "boot", "test", "dry-activate", "reboot"])
                        .required(true),
                ),
        )
        .get_matches();

    let result = match matches.subcommand() {
        ("path", Some(path_matches)) => deploy_path(
            &get_ssh_tool(
                path_matches.value_of("target-host").unwrap(),
                path_matches.value_of("extra-ssh-options"),
                path_matches
                    .value_of("nix-serve-port")
                    .unwrap()
                    .parse()
                    .unwrap(),
                path_matches.is_present("use-remote-sudo"),
            ),
            path_matches
                .value_of("nix-serve-port")
                .unwrap()
                .parse()
                .unwrap(),
            path_matches.is_present("use-substitutes"),
            path_matches.value_of("PATH").unwrap(),
            path_matches.value_of("signing-key"),
            path_matches.value_of("store"),
            path_matches.value_of("profile-path"),
        ),
        ("system", Some(system_matches)) => deploy_system(
            &get_ssh_tool(
                system_matches.value_of("target-host").unwrap(),
                system_matches.value_of("extra-ssh-options"),
                system_matches
                    .value_of("nix-serve-port")
                    .unwrap()
                    .parse()
                    .unwrap(),
                system_matches.is_present("use-remote-sudo"),
            ),
            system_matches
                .value_of("nix-serve-port")
                .unwrap()
                .parse()
                .unwrap(),
            system_matches.is_present("use-substitutes"),
            system_matches.value_of("PATH").unwrap(),
            system_matches.value_of("signing-key"),
            system_matches.value_of("ACTION").unwrap(),
            system_matches.value_of("profile-path").unwrap(),
        ),
        _ => unreachable!(),
    };
    if let Err(e) = result {
        println!("{}", e);
        exit(1);
    }
}
