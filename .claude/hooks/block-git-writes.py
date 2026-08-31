#!/usr/bin/env python3
"""PreToolUse(Bash) guard: deny any git write operation.

Fails closed — only an allowlist of read-only git subcommands is permitted; every
other git subcommand (and any command that mentions git but can't be parsed) is
denied. Stays silent for non-git and read-only-git commands so normal permissioning
is untouched.
"""
import json
import re
import shlex
import sys

READONLY = {
    "status", "log", "show", "diff", "diff-tree", "diff-index", "rev-parse",
    "rev-list", "describe", "blame", "shortlog", "ls-files", "ls-tree",
    "ls-remote", "cat-file", "name-rev", "symbolic-ref", "for-each-ref",
    "whatchanged", "grep", "show-ref", "count-objects", "var", "version",
    "help", "merge-base", "cherry", "show-branch",
}
GLOBAL_OPTS_WITH_ARG = {
    "-C", "-c", "--git-dir", "--work-tree", "--namespace", "--exec-path",
}
# Operators after which the next word starts a new simple command.
CMD_RESET = {";", "&", "&&", "|", "||", "(", ")", "{", "}", "\n", "!"}
# Wrappers that prefix a command; the word after them is still a command.
WRAPPERS = {
    "sudo", "env", "command", "builtin", "nice", "nohup", "time", "xargs",
    "then", "do", "else", "exec", "stdbuf", "setsid",
}
ENV_ASSIGN = re.compile(r"^[A-Za-z_][A-Za-z0-9_]*=")


def git_subcommands(cmd):
    """Return the subcommand token for each git invocation in COMMAND position.

    A bare `git` that appears as an argument to another command (e.g. `grep git`)
    is ignored; `sudo git ...` and `FOO=1 git ...` are still caught.
    """
    lex = shlex.shlex(cmd, posix=True, punctuation_chars=True)
    lex.whitespace_split = True
    try:
        toks = list(lex)
    except ValueError:
        return [None] if re.search(r"\bgit\b", cmd) else []  # unparseable -> fail closed

    subs = []
    at_cmd = True  # first token is in command position
    i = 0
    while i < len(toks):
        t = toks[i]
        if t in CMD_RESET:
            at_cmd = True
            i += 1
            continue
        if at_cmd and (t in WRAPPERS or ENV_ASSIGN.match(t)):
            # stay in command position for the following word
            i += 1
            continue
        if at_cmd and t.rsplit("/", 1)[-1] == "git":
            j = i + 1
            while j < len(toks):
                a = toks[j]
                if a in GLOBAL_OPTS_WITH_ARG:
                    j += 2
                    continue
                if a.startswith("-"):
                    j += 1
                    continue
                break
            subs.append(toks[j] if j < len(toks) and toks[j] not in CMD_RESET else None)
            i = j
            at_cmd = False
            continue
        at_cmd = False
        i += 1
    return subs


def main():
    data = json.load(sys.stdin)
    if data.get("tool_name") != "Bash":
        return
    cmd = (data.get("tool_input") or {}).get("command", "")
    offending = [s for s in git_subcommands(cmd) if s is None or s not in READONLY]
    if offending:
        print(json.dumps({"hookSpecificOutput": {
            "hookEventName": "PreToolUse",
            "permissionDecision": "deny",
            "permissionDecisionReason":
                "git write operations are disabled in this project; only read-only "
                "git subcommands are permitted. Offending: " + repr(offending),
        }}))


main()
