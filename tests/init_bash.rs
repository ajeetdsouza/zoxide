//! Regression tests for Bash prompt-hook installation.
#![cfg(unix)]

use assert_cmd::Command;

#[test]
fn bash_prompt_hook_is_installed_once() {
    let output = Command::cargo_bin("zoxide")
        .unwrap()
        .args(["init", "bash", "--hook", "prompt"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let init = String::from_utf8(output.stdout).unwrap();

    for prompt in [
        "unset PROMPT_COMMAND",
        "PROMPT_COMMAND=''",
        "PROMPT_COMMAND=':'",
        "PROMPT_COMMAND=':;__zoxide_hook'",
        "PROMPT_COMMAND=()",
        "PROMPT_COMMAND=(':')",
        "PROMPT_COMMAND=(':' '__zoxide_hook')",
        "PROMPT_COMMAND=('__zoxide_hook' ':')",
        "PROMPT_COMMAND=([2]=':' [7]='__zoxide_hook')",
    ] {
        let script = format!(
            r#"
{prompt}
eval "$ZOXIDE_INIT"
eval "$ZOXIDE_INIT"
calls=0
__zoxide_hook() {{ calls=$((calls + 1)); }}
for command in "${{PROMPT_COMMAND[@]}}"; do
    eval "$command"
done
[[ $calls -eq 1 ]] || {{ printf 'hook ran %s times\n' "$calls" >&2; exit 1; }}
"#
        );
        Command::new("bash")
            .env("ZOXIDE_INIT", &init)
            .args(["--noprofile", "--norc", "-c", &script])
            .assert()
            .success()
            .stdout("")
            .stderr("");
    }
}
