# Kensaku

Kensaku (検索, “search”) is a Linux terminal system-information tool with fractal art. It reads local system information and renders it alongside an ASCII fractal. A Nerd Font is recommended for the icons.

## Install and run

Build with a Rust toolchain and Cargo:

```sh
cargo build --release --locked
./target/release/kensaku
```

Alternatively, run the default package from the Nix flake with `nix run .`. The flake also exports `homeManagerModules.default`. Flake inputs are pinned in `flake.lock`; use `nix flake update` to refresh them.

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
fractal = "random"   # a name below, or "none"; see kensaku --list-fractals
max_length = 50      # columns, 1–500
max_breadth = 20     # rows, 1–200
# seed = 42          # reproducible choice of fractal and random parameters
# depth = 3          # geometric fractal iterations, 1–6
# max_iterations = 80 # escape-time iterations, 1–1000

# Parameterized fractals read their parameters from a table named after them.
# Each parameter accepts five forms — constant, "random", a range, a set-op
# expression, or a finite set:
[art.julia]
c = [-0.8, 0.156]     # constant [real, imaginary]
# c = "random"                       # default: a point on the main cardioid
# c = [[-1, -1], [1, 1]]              # a rectangle, real ∈ [-1,1], imaginary ∈ [-1,1]
# c = [[[-1,-1],[1,1]], "difference", [[-0.25,-0.25],[0.25,0.25]]]
# c = { one_of = [[-0.8, 0.156], [0.285, 0.01]] }

[art.multibrot]
power = [[2, 3], "union", [6, 8]]  # 2..=3 ∪ 6..=8
# power = 3          # or "random" (2..=8), [2, 6], { one_of = [3, 5, 7] }
```

Use `kensaku --list-fractals` to list the available names, and `kensaku --help` for command-line usage. The default, `random`, chooses a fractal each run. Set `seed` to reproduce both the selection and random parameters. Choose `none` or omit `[art]` to disable art. `network` remains an alias for `ip` in older configurations.

Older configuration names changed: `mandelbrot_set` → `mandelbrot`, `julia_set` → `julia`, and `julia_c`/`multibrot_power` moved to `[art.julia].c`/`[art.multibrot].power`.

The geometric fractals are finite approximations of examples in Wikipedia's [list of fractals by Hausdorff dimension](https://en.wikipedia.org/wiki/List_of_fractals_by_Hausdorff_dimension):

| Configuration name | Fractal | Limiting Hausdorff dimension |
| --- | --- | --- |
| `cantor_dust` | Two-dimensional Cantor dust | log(4) / log(3) ≈ 1.262 |
| `koch_curve` | Koch curve | log(4) / log(3) ≈ 1.262 |
| `vicsek` | Vicsek cross | log(5) / log(3) ≈ 1.465 |
| `sierpinski_triangle` | Sierpiński triangle | log(3) / log(2) ≈ 1.585 |
| `sierpinski_carpet` | Sierpiński carpet | log(8) / log(3) ≈ 1.893 |

There are two parameterized fractals, whose parameters live in a sibling table named after them (`[art.julia]`, `[art.multibrot]`). A parameter table for a fractal that is not selected is an error, except under `fractal = "random"`, where it applies when that fractal is drawn.

| Fractal | Section | Parameter | Default | Standard random domain |
| --- | --- | --- | --- | --- |
| `julia` | `[art.julia]` | `c` | `"random"` | the boundary of the Mandelbrot set's main cardioid |
| `multibrot` | `[art.multibrot]` | `power` | `3` | integers 2 through 8 |

All other fractals take no parameters. A parameter accepts five forms:

- **Constant** — `c = [-0.8, 0.156]`, or `power = 5`.
- **Random** — `c = "random"` uses the parameter's standard random domain. For Julia this is a random point on the boundary of the Mandelbrot set's main cardioid, uniform in the cardioid parameter θ (the default when `[art.julia]` or `c` is omitted); for Multibrot `power` it is a uniform integer from 2 through 8.
- **Range** — `c = [[-1, -1], [1, 1]]` samples uniformly from the rectangle `real ∈ [-1,1]`, `imaginary ∈ [-1,1]`. For a discrete parameter, `power = [2, 6]` samples the integers 2 through 6.
- **Set operations** — ranges alternate with `"union"`, `"intersection"`, or `"difference"` and fold left-to-right: `c = [[[-1,-1],[1,1]], "difference", [[-0.25,-0.25],[0.25,0.25]]]` cuts a hole in the rectangle. `A, op, B, op, C` means `(A op B) op C`; a domain whose set operations leave no points is rejected.
- **Finite set** — `c = { one_of = [...] }` (or `power = { one_of = [3, 5, 7] }`) picks one value uniformly. Note the braces: a bare array already means "range", so a set uses the `one_of` key.

For `multibrot`, `power` replaces the exponent in z → zᵖ + c; valid values are the integers 2 through 8 (default 3). A `seed` makes every random draw repeatable, including `fractal = "random"` selection: constants are used verbatim, while the range, set-operation, and finite-set forms draw one value from their domain each run, deterministically under a fixed seed. Seeded output is repeatable with the same Kensaku and dependency versions and may change after upgrades. `max_iterations` changes the detail of all escape-time fractals, including Julia and Multibrot; higher values take longer.

These dimensions describe the infinite mathematical sets, not the finite character images. `depth` controls their iteration count (default 3); detail can disappear at high depths on a small grid. The other choices are `mandelbrot`, `julia`, `burning_ship`, `tricorn`, and `multibrot`; `depth` does not affect these escape-time images.

Disk usage is for the filesystem containing `/`, as reported by `df -Pk`, and uses binary units. Memory and swap are from `/proc/meminfo`; load is the 1-, 5-, and 15-minute load average. CPU threads reflect the parallelism available to the process. Local IP is the address selected for an IPv4 route to `8.8.8.8`; it may be unavailable when no route exists, and is not a public-IP lookup. Package counts include each available supported manager (pacman, dpkg, RPM, and Flatpak applications); installed managers that fail are omitted. Some fields depend on Linux interfaces, system commands, hardware, or session environment variables and show `Unavailable` when they cannot be determined.

For Home Manager, import the flake's module and set `programs.kensaku.enable = true`. Optional `programs.kensaku.settings` contains the same keys as the TOML example. With no settings, the application uses its defaults.

## Development

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --locked
```
