Rupes is a tool to scan all files within a directory and find any files that have multiple identical copies.

## Usage
`rupes [OPTIONS] [DIRECTORY]`

### Options
```
  -r, --recursive              Recursively search directory
  -e, --exclude-dots           Exclude files and directories that begin with '.'
  -f, --filter <FILTER>        Filter files by pattern, only files with names matching this pattern will be included
  -l, --follow-symlinks        Follow symlinks, by default symbolic links are ignored
  -5, --md5                    Use Md5 instead of Sha256, speeds up duplication detection but increases risk of collision drastically
  -M, --max <MAX>              Maximum file size allowed in bytes, larger files will be skipped
  -m, --min <MIN>              Minimum file size allowed in bytes, smaller files will be skipped
  -q, --quiet                  Hide progress information
  -1, --separator <SEPARATOR>  Character to separate duplicate file paths with [default: "\n"]
  -t, --time                   See total execution time of rupes
  -s, --size                   Display the amount of space wasted by each group of duplicate files
  -S, --total-size             Display the total amount of space wasted by duplicate files
  -d, --details                Display all details, equivalent of appending -sSt to command
  -V, --version                Print rupes version
  -h, --help                   Print help
```

### Examples
Search your cwd recursively for duplicate files and get the total amount of space wasted
```shell
rupes -rS
```

Search a directory for any duplicated txt files
```shell
rupes -f '^.+[.]txt$' /path/to/directory
```

Search a directory recursively for duplicate files but ignore dotfiles
```shell
rupes -re /path/to/directory
```

Search a directory recursively for duplicate files, ignoring dotfiles, showing all details, and using Md5 for hashing
```shell
rupes -red5 /path/to/directory
```