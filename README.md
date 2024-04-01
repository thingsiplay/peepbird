# Peepbird

Lookup count of unread Thunderbird mails.

- Author: [Tuncay D.](https://github.com/thingsiplay)
- Source: [Github](https://github.com/thingsiplay/peepbird)
- Download: [Github Releases](https://github.com/thingsiplay/peepbird/releases)
- License: [MIT](LICENSE)

## What is this program for?

`peepbird` is a simple, fast and lightweight standalone CLI application for
Thunderbird written in Rust. It's only job is to get the number of unread
emails from any given mailbox account (.msf-files) and print the sum to stdout.
The operation and output is basically instantaneous (`time peepbird` usually
results in `0m0,001s` execution time on my modern desktop computer).

## Installation

Download binary directly from Releases or compile from source.

### Requirements

Tested and build for Linux only. There is no runtime dependency; not even
`glibc`, as it's build with `musl` by default. Thunderbird is not required to be
running or even installed on the system, as only the mailbox FILES are read.

### Option 1: Download from Releases

The following commands will download from
[Releases](https://github.com/thingsiplay/peepbird/releases) page the latest
package, unpack and make the program executable. From here you could in example
put it in one of the directories in your system `$PATH` .

```sh
curl -LO "https://github.com/thingsiplay/peepbird/releases/latest/download/peepbird-linux.tar.gz"
tar -xf peepbird-linux.tar.gz
chmod +x peepbird
```

Test it with:

```sh
./peepbird --version
```

### Option 2: Build from source with Cargo

If you have Rust and Cargo tools installed, then you can also compile directly
from source. At default `musl` is used to build the program (instead of
`glibc`) to avoid any dependency. Install `musl` target for your Rust build
environment with:

```sh
rustup target add "x86_64-unknown-linux-musl"
```

Then download and build the program itself:

```sh
git clone "https://github.com/thingsiplay/peepbird"
cd peepbird
cargo build --release
```

After the build process completed succesfully, the resulting `peepbird` binary
should be found under `./target/x86_64-unknown-linux-musl/release/` . Run it
with:

```sh
cargo run --release -- -h
```

Or directly with:

```sh
./target/x86_64-unknown-linux-musl/release/peepbird -h
```

## How to use

```sh
Usage: peepbird [OPTIONS] [FILES]...

Usage: peepbird [-p DIR] [-c FILE] [-C]
                [-z] [-n] [-b TEXT] [-a TEXT] [-l]
                [-d] [-h] [-V]
```

This is a commandline application without graphical interface. The most basic
operation is to give it one or more paths to Thunderbird mailbox FILES or
folders. (In the below examples replace the "xxxxxxx" with your real profile
name.)

A single mailbox path as input file:

```sh
peepbird ~/.thunderbird/xxxxxxx.default/ImapMail/imap.googlemail.com/INBOX.msf
```

... is equivalent to:

```sh
peepbird --profile ~/".thunderbird/xxxxxxx.default" "ImapMail/imap.googlemail.com/INBOX.msf"
```

If the mailbox FILES are relative paths, then `peepbird` will search them in
the users Thunderbird profile directory. If no profile is given or configured,
then it will try to find the default profile. You can also omit the inbox
filename:

```sh
peepbird ImapMail/imap.googlemail.com
```

In which case the mailbox folder will be searched for filenames "INBOX.msf" or
"Inbox.msf" and used as input path. In the above example we also did not
specify any profile folder either, which instructs to lookup default
Thunderbird profile from `~/.thunderbird/profile.ini` (if this setting is not
found in users config file at `~/.config/peepbird/options.toml`).

### Examples

Note: In the below examples, the Dollar sign `$` represents anything after it
as an user command to enter, followed by a possible output from the program.

```sh
# Just output the number of unread messages.
$ peepbird
0
```

```sh
# In addition to the total sum, list each mailbox path and their individual
# count of unread messages too. At default the user configuration file is read
# from `~/.config/peepbird/options.toml` , which in our case includes some
# mailbox FILES.
$ peepbird --location
3 /home/tuncay/.thunderbird/xxxxxxx.default/ImapMail/imap.googlemail.com/INBOX.msf
1 /home/tuncay/.thunderbird/xxxxxxx.default/Mail/pop3.live.com/Inbox.msf
4
```

```sh
# Focus on one specific mailbox only. Make use of automatic expansion of
# profile folder and filenames.
$ peepbird -l "ImapMail/imap.googlemail.com"
3 /home/tuncay/.thunderbird/xxxxxxx.default/ImapMail/imap.googlemail.com/INBOX.msf
3
```

```sh
# Show an icon with a space before the number. If count is '0', then hide
# number and show icon only. Also strip the additional space if number is hidden.
$ peepbird -b"📪 " --no-zero --trim
📪
```

```sh
# Exclude user config file and force using a specific Thunderbird profile.
# Without input mailbox FILES an error will be displayed.
$ peepbird --no-config --profile ~/.thunderbird/xxxxxxx.default/
Error: No input files for mailboxes specified.

# Return value will be set accordingly. `0` indicates success, otherwise a failure.
$ echo "$?"
1
```

## Configuration

Default settings can be configured at `~/.config/peepbird/options.toml` in
[TOML](https://toml.io/) format. It's recommended to specify user profile and
at least one mailbox. The options are the same from help listing at `peepbird
-h` . You don't have to include or specify all options, only those you care
about.

An example `~/.config/peepbird/options.toml`:

```toml
files = [
    "ImapMail/imap.googlemail.com/INBOX.msf",
    "Mail/pop3.live.com",
]
profile = "~/.thunderbird/xxxxxxx.default"
dump_config = false
no_config = false
no_zero = true
no_newline = false
trim = false
before = "📪"
after = ""
location = true
```

Commandline options still have higher priority over any defaults or
configuration file settings. To completely disable this config file, use option
`-C` or `--no-config` on commandline, to rely on commandline options only.

### How to find my profile and mailbox?

The simplest way would be to search with a shell command to quickly and
automatically find a list of your Thunderbird mailbox FILES.

```sh
find ~/.thunderbird -name INBOX.msf -or -name Inbox.msf
```

Each line from the `find` result should contain a mailbox FILE path. You can
directly copy and use them as input arguments for `peepbird`.

#### Search manually

Alternatively you can also lookup those mailbox folders through Thunderbird
itself. This has the advantage to easily identify which account belongs to
which folder, as otherwise it can be confusing if you have many profiles or
mailboxes setup:

- open Thunderbird
- go to the menu "Account Settings"
- each of your accounts have a "Server Settings", open that page for the email
  account you want to use
- at the bottom of the page is "Local Directory" listed, which is the main
  folder for the selected account
- right mouse click that field and copy the directory path
- clipboard should contain something like:
  `/home/tuncay/.thunderbird/xxxxxxx.default/Mail/pop3.live.com`
- this directory includes multiple \*.msf files
- either choose one of the filenames (in example `Inbox.msf` or `INBOX.msf`) to
  add to the directory path, in example
  `/home/tuncay/.thunderbird/xxxxxxx.default/Mail/pop3.live.com/Inbox.msf`
- or just use the directory itself as input FILE, as `peepbird` will search and
  add an inbox name itself

## Example Setups

Here are some ideas of where or how to use the program.

### Fly script

A little script to start Thunderbird if it's not running, otherwise change
focus to Thunderbird window if any unread mail is found. Intended use case
might be some custom setup where on a mouse click in example would run the
script.

```bash
#!/usr/bin/env bash

if ! pidof -q thunderbird; then
	thunderbird -mail </dev/null &>/dev/null &
	disown
else
	if [ "$(peepbird -C "Mail/smart mailboxes")" -gt 0 ]; then
		thunderbird -mail </dev/null &>/dev/null
	fi
fi
```

### KDE Plasma widget

The program can be used in various places, such as bars for periodic checking.
There is a user created widget for Plasma, which you need to install in order
to follow this suggested setup:
[Command Output \[Plasma 6\]](https://www.pling.com/p/2136636/)

Just click "+ Add Widget..." on your panel, click the "Get New Widgets..." and
"Download New Plasma Widgets". In the new window search for "Command Output
\[Plasma 6\]" (note, there is an older one for Plasma 5, don't get mixed up)
and install it.

Open the configuration dialog for the widget. I use following settings for this
widget.

#### Command Tab

- Command: `peepbird -ztb"📪 "`
- Run every: `300000` (note: ms, which is every 5 minutes)
- Wait for completion: `[ ]` Enabled (note: checkbox is empty, so its disabled)

#### Actions Tab

- Hover Command: `peepbird --version`
- Run Command: `peepbird -ztb"📪 "`

### Thunderbird smart mailboxes

Thunderbird has a functionality that is called smart mailboxes. This is
basically a virtual mailbox that combines other sources into a single one. This
is my preferred way of using Thunderbird and I want to show you how to
configure it. This can even include Blogs & News Feeds.

- open Thunderbird
- click the main menu (hamburger menu)
- go to the submenu `View` > `Folders` and enable `Unified Folders`
- on the left pane of Thunderbird, with the list of mailboxes right click on
  `Inbox` for the Unified Folders and choose `Properties`
- a new dialog should popup with the Name: `Inbox on Unified Folders`
- click `Choose...` button, in this new dialog we can select each Inbox (or
  anything else) of any mailbox account from our profile
- I simply click all checkboxes on `Inbox` for each account, plus add Blogs &
  News-Feeds
- accept with click on `OK` button and then accept again with `Update` button

After that, you can run `peepbird` with this single input FILE only with
`peepbird "Mail/smart mailboxes"` .
