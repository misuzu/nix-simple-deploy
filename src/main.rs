use clap::*;
use cmd_lib::run_cmd;
use std::io;
use std::process::exit;

fn deploy_path(
    path: &str,
    signing_key: &str,
    target_host: &str,
    use_substitutes: bool,
    use_local_sudo: bool,
) -> io::Result<()> {
    let sign_tool = if use_local_sudo {
        "sudo nix sign-paths"
    } else {
        "nix sign-paths"
    };
    let copy_tool = if use_substitutes {
        "nix copy -s"
    } else {
        "nix copy"
    };
    run_cmd!("{} -r -k {} {}", sign_tool, signing_key, path)?;
    run_cmd!("{} --to ssh://{} {}", copy_tool, target_host, path)?;

    Ok(())
}

fn deploy_system(
    path: &str,
    target_host: &str,
    use_remote_sudo: bool,
    action: &str,
    profile: &str,
) -> io::Result<()> {
    let ssh_tool = if use_remote_sudo {
        format!("ssh {} sudo", target_host)
    } else {
        format!("ssh {}", target_host)
    };
    let profile_path = match action {
        "switch" | "boot" | "reboot" => {
            if profile == "system" {
                "/nix/var/nix/profiles/system".to_string()
            } else {
                format!("/nix/var/nix/profiles/system-profiles/{}", profile)
            }
        }
        _ => path.to_string(),
    };
    if action != "test" && action != "dry-activate" {
        run_cmd!("{} nix-env -p {} --set {}", ssh_tool, profile_path, path)?;
    }
    let remote_action = if action == "reboot" { "boot" } else { action };
    run_cmd!(
        "{} {}/bin/switch-to-configuration {}",
        ssh_tool,
        profile_path,
        remote_action
    )?;
    if action == "reboot" {
        run_cmd!("{} reboot", ssh_tool)?;
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
                    Arg::with_name("signing-key")
                        .short("k")
                        .long("signing-key")
                        .help("File containing the secret signing key")
                        .value_name("/path/to/signing-key")
                        .required(true),
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
                    Arg::with_name("use-local-sudo")
                        .long("use-local-sudo")
                        .help(
                            "When set, nix-simple-deploy prefixes local commands \
                             that requires privileges with sudo. \
                             Setting this option allows deploying using local non-root user",
                        ),
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
                    Arg::with_name("signing-key")
                        .short("k")
                        .long("signing-key")
                        .help("File containing the secret signing key")
                        .value_name("/path/to/signing-key")
                        .required(true),
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
                    Arg::with_name("use-local-sudo")
                        .long("use-local-sudo")
                        .help(
                            "When set, nix-simple-deploy prefixes local commands \
                             that requires privileges with sudo. \
                             Setting this option allows deploying as a non-root user",
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
                    Arg::with_name("profile")
                        .short("p")
                        .long("profile")
                        .help("Profile name")
                        .default_value("system")
                        .required(true),
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
            path_matches.value_of("PATH").unwrap(),
            path_matches.value_of("signing-key").unwrap(),
            path_matches.value_of("target-host").unwrap(),
            path_matches.is_present("use-substitutes"),
            path_matches.is_present("use-local-sudo"),
        ),
        ("system", Some(system_matches)) => deploy_path(
            system_matches.value_of("PATH").unwrap(),
            system_matches.value_of("signing-key").unwrap(),
            system_matches.value_of("target-host").unwrap(),
            system_matches.is_present("use-substitutes"),
            system_matches.is_present("use-local-sudo"),
        )
        .and_then(|_| {
            deploy_system(
                system_matches.value_of("PATH").unwrap(),
                system_matches.value_of("target-host").unwrap(),
                system_matches.is_present("use-remote-sudo"),
                system_matches.value_of("ACTION").unwrap(),
                system_matches.value_of("profile").unwrap(),
            )
        }),
        _ => unreachable!(),
    };
    if let Err(e) = result {
        println!("Error occured while running: {}", e);
        exit(1);
    }
}
