use rio_backend::config::Shell;

/// WSL needs its Linux cwd in argv, not in CreateProcessW's Windows cwd.
/// Keep launch options and the command following --exec/-- intact.
pub(super) fn inherit_directory(shell: &mut Shell, path: &str) -> bool {
    let Some(program) = shell.program.as_deref() else {
        return false;
    };
    let name = program.rsplit(['/', '\\']).next().unwrap_or(program);
    if !name.eq_ignore_ascii_case("wsl.exe") && !name.eq_ignore_ascii_case("wsl") {
        return false;
    }

    let (distribution, directory) = if let Some(unc) = path.strip_prefix("\\\\") {
        let mut parts = unc.splitn(3, '\\');
        let host = parts.next().unwrap_or_default();
        if !host.eq_ignore_ascii_case("wsl.localhost")
            && !host.eq_ignore_ascii_case("wsl$")
        {
            return false;
        }
        let Some(distribution) = parts.next().filter(|s| !s.is_empty()) else {
            return false;
        };
        let Some(directory) = parts.next() else {
            return false;
        };
        (
            Some(distribution),
            format!("/{}", directory.replace('\\', "/")),
        )
    } else {
        (None, path.to_owned())
    };

    let mut args = Vec::new();
    let mut index = 0;
    while index < shell.args.len() {
        let arg = &shell.args[index];
        // Everything from here on belongs to the Linux command.
        if matches!(arg.as_str(), "--" | "--exec" | "-e") || !arg.starts_with('-') {
            args.extend_from_slice(&shell.args[index..]);
            break;
        }
        if arg == "--cd"
            || (distribution.is_some() && matches!(arg.as_str(), "-d" | "--distribution"))
        {
            index += 2;
            continue;
        }
        args.push(arg.clone());
        index += 1;
        if matches!(arg.as_str(), "-d" | "--distribution" | "-u" | "--user")
            && index < shell.args.len()
        {
            args.push(shell.args[index].clone());
            index += 1;
        }
    }
    let mut prefix = vec!["--cd".into(), directory];
    if let Some(distribution) = distribution {
        prefix.extend(["--distribution".into(), distribution.into()]);
    }
    prefix.extend(args);
    shell.args = prefix;
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wsl_inherits_reported_distribution_and_directory() {
        let mut shell = Shell {
            program: Some(r"C:\Windows\System32\wsl.exe".into()),
            args: [
                "-d",
                "Debian",
                "--cd",
                "~",
                "-u",
                "user",
                "--exec",
                "zsh",
                "--cd",
                "command-arg",
            ]
            .map(str::to_owned)
            .to_vec(),
        };
        assert!(inherit_directory(
            &mut shell,
            r"\\wsl.localhost\Ubuntu\home\user\папка с пробелами"
        ));
        assert_eq!(
            shell.args,
            [
                "--cd",
                "/home/user/папка с пробелами",
                "--distribution",
                "Ubuntu",
                "-u",
                "user",
                "--exec",
                "zsh",
                "--cd",
                "command-arg"
            ]
        );
    }

    #[test]
    fn wsl_handles_drive_paths_and_legacy_root_share() {
        let mut shell = Shell {
            program: Some("wsl".into()),
            args: vec!["--cd".into(), "~".into()],
        };
        assert!(inherit_directory(&mut shell, r"D:\Projects\my project"));
        assert_eq!(shell.args, ["--cd", r"D:\Projects\my project"]);
        assert!(inherit_directory(&mut shell, r"\\wsl$\Ubuntu\"));
        assert_eq!(shell.args, ["--cd", "/", "--distribution", "Ubuntu"]);
    }

    #[test]
    fn other_shells_are_not_rewritten() {
        let mut shell = Shell {
            program: Some("pwsh.exe".into()),
            args: vec![],
        };
        assert!(!inherit_directory(&mut shell, r"C:\Users\user"));
        assert!(shell.args.is_empty());
    }
}
