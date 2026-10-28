# petrovic.foo

My personal site: a Leptos app with server-side rendering on Axum, styled with Tailwind, built with
Nix. The server also compiles my CV from LaTeX and serves it at `/resume`.

## Setup

Everything comes from the flake's dev shell: the Rust toolchain, cargo-leptos, wasm-bindgen,
Tailwind and TeX Live.

```sh
direnv allow    # once; loads the dev shell whenever I cd in (or use `nix develop`)
```

Open the folder in VSCodium and accept the recommended extensions. The direnv extension has to
load the shell before rust-analyzer starts, so reload the window if it asks.

## Working on the site

```sh
cargo leptos watch    # dev server on http://127.0.0.1:3000, live reload on save
```

- Pages and components live in `src/app.rs`. Saving reformats `view!` macros with leptosfmt.
- Styles are Tailwind classes written inline. `style/tailwind.css` is only the entry point.
- Files in `public/` are served from the site root.

Stop the dev server with Ctrl+C. If a server outlives its watch and port 3000 stays taken:

```sh
pkill -f target/debug/petrovic_foo
```

## Editing my CV

The source is `public/resume.tex`. There are two ways to see changes:

- **On the site:** save, then refresh `http://127.0.0.1:3000/resume`. The server recompiles the
  PDF only when the `.tex` has changed, and serves a cached copy otherwise.
- **In the editor:** LaTeX Workshop builds on save (or manually with `Ctrl+Alt+B`) and previews
  with `Ctrl+Alt+V`. Its output goes to `build/`, which is gitignored.

Keep it to one page. If I add a LaTeX package, I also add it to `tex` in `flake.nix`, or the
server's build will fail.

`/resume.tex` is downloadable too, because everything in `public/` is served.

## Building and running for real

```sh
nix build            # release server + site in ./result
nix run              # build and start it on 127.0.0.1:3000
nix build .#resume   # just the CV as a PDF
```

The packaged server brings its own TeX Live, so the host doesn't need LaTeX installed.

Settings, all optional:

| Variable           | Default                    | What it does                                   |
| ------------------ | -------------------------- | ---------------------------------------------- |
| `LEPTOS_SITE_ADDR` | `127.0.0.1:3000`           | Address the server listens on                  |
| `RESUME_CACHE_DIR` | `/tmp/petrovic_foo-resume` | Where compiled PDFs are cached across restarts |
| `RESUME_TEX`       | `<site root>/resume.tex`   | Use a different `.tex` file                    |

## Upgrading

- **Rust:** `rust-toolchain.toml` tracks stable, so `nix flake update` picks up the latest.
- **wasm-bindgen:** the crate in `Cargo.toml` is pinned to the exact version of `wasm-bindgen-cli`
  in `flake.nix`. cargo-leptos fails if they differ, so bump both together.
