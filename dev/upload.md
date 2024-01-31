# Upload Command

**_Usage_**: `clift upload`
**Argument**: site (Optional String)

This command uploads the fastn package to `ft`. 

There are two ways to call this command.

1. Without `site` parameter

    ```shell
    clift upload
    ```
    This checks the `fastn.package` value of FASTN.ftd in the current package and
    then pass it as the `site` value.


2. With `site` parameter

    ```shell
    clift upload <site>
    ```
    This provides option to pass custom `site` value.

## How it works

It takes `site` as the `fastn.package` name and calls the 
`api.fifthtry.com/api/all-files/?site=<site>` API to fetch all the files and
content present in the package in `ft`. Then it compares with the files present
in the file system and calls the `api.fifthtry.com/api/upload/`. This uploads 
the files and returns if there's a conflict?
