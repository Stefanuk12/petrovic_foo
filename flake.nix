{
  description = "petrovic.foo - personal site (Leptos SSR + Axum)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      nixpkgs,
      flake-utils,
      rust-overlay,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };
        rust = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
        rustPlatform = pkgs.makeRustPlatform {
          cargo = rust;
          rustc = rust;
        };

        # cargo-leptos uses these from PATH instead of downloading them (downloads don't run on NixOS).
        # wasm-bindgen-cli must match the `wasm-bindgen` version pinned in Cargo.toml.
        leptosTools = with pkgs; [
          cargo-leptos
          wasm-bindgen-cli_0_2_127
          binaryen
          tailwindcss_4
        ];

        # Everything resume.tex needs; the server runs latexmk from this at request time.
        tex = pkgs.texliveSmall.withPackages (
          ps: with ps; [
            latexmk
            titlesec
            marvosym
            enumitem
            fancyhdr
            preprint
            cm-super
          ]
        );

        resume =
          pkgs.runCommand "resume.pdf"
            {
              nativeBuildInputs = [ tex ];
            }
            ''
              cp ${./public/resume.tex} resume.tex
              HOME=$TMPDIR latexmk -pdf -interaction=nonstopmode resume.tex
              cp resume.pdf $out
            '';
      in
      {
        packages.resume = resume;

        packages.default = rustPlatform.buildRustPackage {
          pname = "petrovic_foo";
          version = "0.1.0";
          src = pkgs.lib.cleanSource ./.;
          cargoLock.lockFile = ./Cargo.lock;

          nativeBuildInputs = leptosTools ++ [ pkgs.makeWrapper ];

          buildPhase = ''
            runHook preBuild
            export HOME=$TMPDIR
            cargo leptos build --release
            runHook postBuild
          '';

          doCheck = false;

          installPhase = ''
            runHook preInstall
            mkdir -p $out/bin $out/share/petrovic_foo
            cp target/release/petrovic_foo $out/bin/
            cp -r target/site $out/share/petrovic_foo/site
            wrapProgram $out/bin/petrovic_foo \
              --set-default LEPTOS_OUTPUT_NAME petrovic_foo \
              --set-default LEPTOS_SITE_ROOT $out/share/petrovic_foo/site \
              --set-default LEPTOS_SITE_PKG_DIR pkg \
              --set-default LEPTOS_SITE_ADDR 127.0.0.1:3000 \
              --set-default LEPTOS_ENV PROD \
              --prefix PATH : ${tex}/bin
            runHook postInstall
          '';

          meta.mainProgram = "petrovic_foo";
        };

        devShells.default = pkgs.mkShell {
          packages = [
            rust
            pkgs.leptosfmt
            tex
          ]
          ++ leptosTools;

          shellHook = ''
            echo "dev server: cargo leptos watch  (http://127.0.0.1:3000)"
          '';
        };
      }
    );
}
