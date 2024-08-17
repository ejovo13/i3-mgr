/// Utilties for calling and piping shell commands.
///
///
///
use crate::prelude::Result;
use std::process as proc;
use std::process::Stdio;

pub(crate) fn cmd(args: &[&str]) -> proc::Command {
    let mut cmd = proc::Command::new(args[0]);

    for arg in &args[1..] {
        cmd.arg(arg);
    }

    cmd
}

pub(crate) fn i3_cmd(cmd_strings: &[&str]) -> Result<String> {
    let mut cmds = vec!["i3-msg"];
    cmds.extend(cmd_strings);
    pipe(&mut [&mut cmd(cmds.as_slice())])
}

/// Create a chain of commands that are piped together and extract the std out.
pub(crate) fn pipe(cmds: &mut [&mut proc::Command]) -> Result<String> {
    for i in 0..cmds.len() - 1 {
        cmds[i].stdout(Stdio::piped());
        let stdout = cmds[i].spawn()?.stdout.unwrap();
        cmds[i + 1].stdin(Stdio::from(stdout));

        if i != cmds.len() - 2 {
            cmds[i + 1].stdout(Stdio::piped());
        };
    }

    // Now spawn the last process and return its stdout
    let stdout_str = String::from_utf8(cmds[cmds.len() - 1].output().unwrap().stdout).unwrap();
    Ok(stdout_str)
}

pub(crate) fn move_window_to_workspace(window_id: u64, target_workspace: &str) -> Result<String> {
    i3_cmd(&[
        &format!(r#"[con_id="{}"]"#, window_id),
        "move",
        "workspace",
        target_workspace,
    ])
}
