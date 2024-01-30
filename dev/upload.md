# Upload Command

**_Usage_**: `clift upload`

This command uploads the fastn package to `ft`. 

## How it works

It takes `site` as the `fastn.package` name and calls the 
`api.fifthtry.com/api/all-files/?site=<site>` API to fetch all the files and
content present in the package in `ft`. Then it compares with the files present
in the file system and calls the `api.fifthtry.com/api/upload/`. This uploads 
the files and returns if there's a conflict?
