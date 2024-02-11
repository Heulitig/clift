# clift - ft cli

`clift` is a CLI utility to download or upload fastn packages to / from FifthTry.

This utility should be used for testing on CI systems.

```sh
$ clift --help
Usage: clift [OPTIONS] [COMMAND]

Commands:
  login    Authenticate with FifthTry
  clone    Clone a fastn package from FifthTry (or CR)
  upload   Upload local package to FifthTry
  tunnel   Expose a local fastn instance via FifthTry
  help     Print this message or the help of the given subcommand(s)

Options:
  -v                       Sets the level of verbosity
  -h, --help               Print help
  -V, --version            Print version
```

Copyright 2024, FifthTry, Inc, All Rights Reserved.

## Installation

```sh
source <(curl -fsSL https://www.fifthtry.com/clift.sh)
```

Supported Platforms: Linux, Mac

## Debug

To run in debug mode

```sh
export ACTIONS_ID_TOKEN_REQUEST_URL="url"
export ACTIONS_ID_TOKEN_REQUEST_TOKEN="token"
export DEBUG_USE_TEJAR_FOLDER="${PROJ_ROOT}/debug-tejar-content"
```
