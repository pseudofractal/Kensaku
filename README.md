# Kensaku
Kensaku (検索) meaning "Search" or "Retreive" in Japanese is the nth system information fetching tool made in Rust.\
The only sligth twist it offers is it generates a random fractal as ASCII(Unicode) art.

You should probably be using something like [Fastfetch](https://github.com/fastfetch-cli/fastfetch) if you want something well maintained and feature rich. This was just a hobby project I made over the weekend because I was bored.

Also this only works on arch, because I was too lazy to find a system agnostic way to do things.

## Configuration
It will not display anything unless configured and will panic if a config file is not found in `.config/kensaku/config.toml`.\
An example (my config) config can be found below:

```toml
accent_color = "cyan"
user_host = true
cpu = true
memory = true
uptime = true
os = true
kernel = true
disk = true
network = true
shell = true
packages = true
wm = true

[art]
max_length = 50
max_breadth = 12
fractal = "julia_set"
```

