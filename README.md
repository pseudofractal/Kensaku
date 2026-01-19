# Kensaku

Kensaku (検索, “search”) is a Linux terminal system-information tool with fractal art. It reads local system information and renders it alongside an ASCII fractal. A Nerd Font is recommended for the icons.

## Install and run

Build with a Rust toolchain and Cargo:

```sh
cargo build --release --locked
./target/release/kensaku
```

Alternatively, run the default package from the Nix flake with `nix run .`. The flake also exports `homeManagerModules.default`. The flake inputs are not locked, so their revisions can change between evaluations.

Kensaku uses useful defaults when no configuration file exists. On Linux it reads `$XDG_CONFIG_HOME/kensaku/config.toml`, or `~/.config/kensaku/config.toml` when `XDG_CONFIG_HOME` is unset. An existing file replaces the defaults: set each desired field to `true`. Unknown fields and invalid values are reported as errors.

```toml
accent_color = "cyan" # black, red, green, yellow, blue, magenta, cyan, white, or color256(0..255)
user_host = true
host_model = true
os = true
kernel = true
cpu = true
cpu_threads = true
load = true
memory = true
swap = true
disk = true
uptime = true
ip = true
packages = true
shell = true
terminal = true
wm = true

[art]
fractal = "random"
max_length = 50  # columns, 1–500
max_breadth = 20 # rows, 1–200
# seed = 42      # reproducible random choice and Julia set
# depth = 3      # geometric fractal iterations, 1–6
# julia_c = [-0.8, 0.156] # fixed real and imaginary components
# multibrot_power = 3     # integer 2–8, or "random"
# max_iterations = 80    # escape-time iterations, 1–1000
```

Use `kensaku --list-fractals` to list the available names, and `kensaku --help` for command-line usage. The default, `random`, chooses a fractal each run. Set `seed` to reproduce both the selection and Julia-set parameters. Choose `none` or omit `[art]` to disable art. `network` remains an alias for `ip` in older configurations.

The geometric fractals are finite approximations of examples in Wikipedia's [list of fractals by Hausdorff dimension](https://en.wikipedia.org/wiki/List_of_fractals_by_Hausdorff_dimension):

| Configuration name | Fractal | Limiting Hausdorff dimension |
| --- | --- | --- |
| `cantor_dust` | Two-dimensional Cantor dust | log(4) / log(3) ≈ 1.262 |
| `koch_curve` | Koch curve | log(4) / log(3) ≈ 1.262 |
| `vicsek` | Vicsek cross | log(5) / log(3) ≈ 1.465 |
| `sierpinski_triangle` | Sierpiński triangle | log(3) / log(2) ≈ 1.585 |
| `sierpinski_carpet` | Sierpiński carpet | log(8) / log(3) ≈ 1.893 |

For `julia_set`, `julia_c = [real, imaginary]` fixes the parameter in z → z² + c. Set `julia_c = "random"` or omit it to choose a point on the boundary of the main cardioid on each run. For `multibrot`, `multibrot_power` fixes the integer exponent in z → zᵖ + c (default 3); use `"random"` to choose an integer from 2 through 8. A `seed` makes these random choices repeatable, and also fixes `fractal = "random"` selection. Fixed parameters are used even when a seed is set. These options apply only when their corresponding fractal is selected. `max_iterations` changes the detail of all escape-time fractals, including Julia and Multibrot; higher values take longer.

These dimensions describe the infinite mathematical sets, not the finite character images. `depth` controls their iteration count (default 3); detail can disappear at high depths on a small grid. The other choices are `mandelbrot_set`, `julia_set`, `burning_ship`, `tricorn`, and `multibrot`; `depth` does not affect these escape-time images.

Disk usage is for the filesystem containing `/`, as reported by `df -Pk`, and uses binary units. Memory and swap are from `/proc/meminfo`; load is the 1-, 5-, and 15-minute load average. CPU threads reflect the parallelism available to the process. Local IP is the address selected for an IPv4 route to `8.8.8.8`; it may be unavailable when no route exists, and is not a public-IP lookup. Package counts include each available supported manager (pacman, dpkg, RPM, and Flatpak applications); installed managers that fail are omitted. Some fields depend on Linux interfaces, system commands, hardware, or session environment variables and show `Unavailable` when they cannot be determined.

For Home Manager, import the flake's module and set `programs.kensaku.enable = true`. Optional `programs.kensaku.settings` contains the same keys as the TOML example. With no settings, the application uses its defaults.

## Development

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --locked
```
