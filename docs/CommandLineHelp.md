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

* `--app-type <APP_TYPES>` — Wolfram application types to include

  Possible values:
  - `mathematica`:
    [Wolfram Mathematica](https://www.wolfram.com/mathematica/)
  - `engine`:
    [Wolfram Engine](https://wolfram.com/engine)
  - `desktop`:
    [Wolfram Desktop](https://www.wolfram.com/desktop/)
  - `player`:
    [Wolfram Player](https://www.wolfram.com/player/)
  - `player-pro`:
    [Wolfram Player Pro](https://www.wolfram.com/player-pro/)
  - `finance-platform`:
    [Wolfram Finance Platform](https://www.wolfram.com/finance-platform/)
  - `programming-lab`:
    [Wolfram Programming Lab](https://www.wolfram.com/programming-lab/)
  - `wolfram-alpha-notebook-edition`:
    [Wolfram|Alpha Notebook Edition](https://www.wolfram.com/wolfram-alpha-notebook-edition/)

* `--debug` — Whether to print application information in the verbose Debug format
* `--raw-value <PROPERTY>` — If specified, the value of this property will be written without any trailing newline

  Possible values:
  - `app-type`:
    [`WolframAppType`] value describing the installation
  - `app-directory`
  - `wolfram-version`:
    [`WolframVersion`] value of the installation
  - `installation-directory`:
    [`$InstallationDirectory`] value of the installation
  - `library-link-c-includes-directory`:
    Wolfram *LibraryLink* C includes directory
  - `kernel-executable-path`:
    Location of the [`WolframKernel`] executable
  - `wolfram-script-executable-path`:
    Location of the [`wolframscript`] executable
  - `wstp-compiler-additions-directory`:
    Location of the WSTP SDK 'CompilerAdditions' directory

* `--property <PROPERTIES>` — Properties to output

  Possible values:
  - `app-type`:
    [`WolframAppType`] value describing the installation
  - `app-directory`
  - `wolfram-version`:
    [`WolframVersion`] value of the installation
  - `installation-directory`:
    [`$InstallationDirectory`] value of the installation
  - `library-link-c-includes-directory`:
    Wolfram *LibraryLink* C includes directory
  - `kernel-executable-path`:
    Location of the [`WolframKernel`] executable
  - `wolfram-script-executable-path`:
    Location of the [`wolframscript`] executable
  - `wstp-compiler-additions-directory`:
    Location of the WSTP SDK 'CompilerAdditions' directory

* `--all-properties` — If set, all available properties will be printed
* `--format <FORMAT>`

  *Possible Values:* `text`, `csv`




## `wolfram-app-discovery list`

List all locatable Wolfram apps

**Usage:** `wolfram-app-discovery list [OPTIONS]`

###### **Options:**

* `--app-type <APP_TYPES>` — Wolfram application types to include

  Possible values:
  - `mathematica`:
    [Wolfram Mathematica](https://www.wolfram.com/mathematica/)
  - `engine`:
    [Wolfram Engine](https://wolfram.com/engine)
  - `desktop`:
    [Wolfram Desktop](https://www.wolfram.com/desktop/)
  - `player`:
    [Wolfram Player](https://www.wolfram.com/player/)
  - `player-pro`:
    [Wolfram Player Pro](https://www.wolfram.com/player-pro/)
  - `finance-platform`:
    [Wolfram Finance Platform](https://www.wolfram.com/finance-platform/)
  - `programming-lab`:
    [Wolfram Programming Lab](https://www.wolfram.com/programming-lab/)
  - `wolfram-alpha-notebook-edition`:
    [Wolfram|Alpha Notebook Edition](https://www.wolfram.com/wolfram-alpha-notebook-edition/)

* `--debug` — Whether to print application information in the verbose Debug format
* `--property <PROPERTIES>` — Properties to output

  Possible values:
  - `app-type`:
    [`WolframAppType`] value describing the installation
  - `app-directory`
  - `wolfram-version`:
    [`WolframVersion`] value of the installation
  - `installation-directory`:
    [`$InstallationDirectory`] value of the installation
  - `library-link-c-includes-directory`:
    Wolfram *LibraryLink* C includes directory
  - `kernel-executable-path`:
    Location of the [`WolframKernel`] executable
  - `wolfram-script-executable-path`:
    Location of the [`wolframscript`] executable
  - `wstp-compiler-additions-directory`:
    Location of the WSTP SDK 'CompilerAdditions' directory

* `--all-properties` — If set, all available properties will be printed
* `--format <FORMAT>`

  *Possible Values:* `text`, `csv`




## `wolfram-app-discovery inspect`

Print information about a specified Wolfram application

**Usage:** `wolfram-app-discovery inspect [OPTIONS] <APP_DIR>`

###### **Arguments:**

* `<APP_DIR>`

###### **Options:**

* `--raw-value <PROPERTY>` — If specified, the value of this property will be written without any trailing newline

  Possible values:
  - `app-type`:
    [`WolframAppType`] value describing the installation
  - `app-directory`
  - `wolfram-version`:
    [`WolframVersion`] value of the installation
  - `installation-directory`:
    [`$InstallationDirectory`] value of the installation
  - `library-link-c-includes-directory`:
    Wolfram *LibraryLink* C includes directory
  - `kernel-executable-path`:
    Location of the [`WolframKernel`] executable
  - `wolfram-script-executable-path`:
    Location of the [`wolframscript`] executable
  - `wstp-compiler-additions-directory`:
    Location of the WSTP SDK 'CompilerAdditions' directory

* `--property <PROPERTIES>` — Properties to output

  Possible values:
  - `app-type`:
    [`WolframAppType`] value describing the installation
  - `app-directory`
  - `wolfram-version`:
    [`WolframVersion`] value of the installation
  - `installation-directory`:
    [`$InstallationDirectory`] value of the installation
  - `library-link-c-includes-directory`:
    Wolfram *LibraryLink* C includes directory
  - `kernel-executable-path`:
    Location of the [`WolframKernel`] executable
  - `wolfram-script-executable-path`:
    Location of the [`wolframscript`] executable
  - `wstp-compiler-additions-directory`:
    Location of the WSTP SDK 'CompilerAdditions' directory

* `--all-properties` — If set, all available properties will be printed
* `--format <FORMAT>`

  *Possible Values:* `text`, `csv`

* `--debug` — Whether to print application information in the verbose Debug format



<hr/>

<small><i>
    This document was generated automatically by
    <a href="https://crates.io/crates/clap-markdown"><code>clap-markdown</code></a>.
</i></small>

