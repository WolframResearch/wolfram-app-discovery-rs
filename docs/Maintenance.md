# Maintenance

This document describes steps required to maintain the `wolfram-app-discovery` project.

### `wolfram-app-discovery` command-line executable help text

This maintenance task should be run every time the `wolfram-app-discovery` command-line
interface changes.

The [`CommandLineHelp.md`](./CommandLineHelp.md) file contains the `--help` text for the
`wolfram-app-discovery` command-line tool. Storing this overview of the help text in a
markdown file makes the functionality of `wolfram-app-discovery` more discoverable, and
serves as an informal "cheet sheet" / reference material. Creation of the contents of
`CommandLineHelp.md` is partially automated by the undocumented `print-all-help`
subcommand.

To update [`CommandLineHelp.md`](./CommandLineHelp.md), execute the following
command:

```
$ cargo run --features=cli -- print-all-help --markdown > docs/CommandLineHelp.md
```

If the content has changed, commit it with a commit message like:
`chore: Regenerate CommandLineHelp.md`.
