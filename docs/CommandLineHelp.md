# Command-Line Help for `wolfram-app-discovery`

This document contains the help content for the `wolfram-app-discovery` command-line program.

**Command Overview:**

* [`wolfram-app-discovery`↴](#wolfram-app-discovery)
* [`wolfram-app-discovery default`↴](#wolfram-app-discovery-default)
* [`wolfram-app-discovery list`↴](#wolfram-app-discovery-list)
* [`wolfram-app-discovery inspect`↴](#wolfram-app-discovery-inspect)

## `wolfram-app-discovery`

Find local installations of the Wolfram Language and Wolfram apps

**Usage:** `wolfram-app-discovery <COMMAND>`

###### **Subcommands:**

* `default` — Print the default Wolfram app
* `list` — List all locatable Wolfram apps
* `inspect` — Print information about a specified Wolfram application



## `wolfram-app-discovery default`

Print the default Wolfram app.

This method uses [`WolframApp::try_default()`] to locate the default app.

**Usage:** `wolfram-app-discovery default [OPTIONS]`

###### **Options:**

* `--app-type` — Wolfram application types to include

  *Possible Values:* `mathematica`, `engine`, `desktop`, `player`, `player-pro`, `finance-platform`, `programming-lab`, `wolfram-alpha-notebook-edition`

* `--debug` — Whether to print application information in the verbose Debug format



## `wolfram-app-discovery list`

List all locatable Wolfram apps

**Usage:** `wolfram-app-discovery list [OPTIONS]`

###### **Options:**

* `--app-type` — Wolfram application types to include

  *Possible Values:* `mathematica`, `engine`, `desktop`, `player`, `player-pro`, `finance-platform`, `programming-lab`, `wolfram-alpha-notebook-edition`

* `--debug` — Whether to print application information in the verbose Debug format



## `wolfram-app-discovery inspect`

Print information about a specified Wolfram application

**Usage:** `wolfram-app-discovery inspect [OPTIONS] <APP_DIR>`

###### **Arguments:**

* `<APP_DIR>`

###### **Options:**

* `--debug` — Whether to print application information in the verbose Debug format



<hr/>

<small><i>
    This document was generated automatically by
    <a href="https://crates.io/crates/clap-markdown"><code>clap-markdown</code></a>.
</i></small>

