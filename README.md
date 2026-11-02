# git_dir

git_dir is a simple CLI utility to download folders from github repos.

## Example:
```bash
git_dir \
    -s "https://github.com/KhronosGroup/glTF-Sample-Assets/tree/main/Models/Fox" \
    -d "/home/irving_wash/Downloads"

ls ~/Downloads
> KhronosGroup-glTF-Sample-Assets-Models_Fox.zip
```

## Installation

```bash
cargo install git_dir
```
