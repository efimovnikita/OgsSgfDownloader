## Online go server SGF downloader

Right now, this tool only download 9x9 sgf's.

### Hot to use it
Extract files from [release](https://github.com/efimovnikita/OgsSgfDownloader/releases) archive, open folder in terminal and type:

```shell
SGFdownloader -p 64817 126739 2172 -r 1 50 --path "/home/maskedball/9x9SGF"
```

### Description and options

```shell
Description:
  Uploader of sgf files from the OGS server

Usage:
  SGFdownloader [options]

Options:
  -p, --players <players> (REQUIRED)  Player id from OGS server. Example: -p 64817 126739
  -r, --range <range> (REQUIRED)      Range of downloadable games. Example: -r 1 100
  --path <path> (REQUIRED)            Folder for saving sgf files. Example: --path '/home/maskedball/9x9SGF'
  --version                           Show version information
  -?, -h, --help                      Show help and usage information
```
