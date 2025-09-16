{
  description = "Development Nix flake for OpenAI Codex CLI";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { nixpkgs, flake-utils, rust-overlay, ... }: 
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };
        pkgsWithRust = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };
        monorepo-deps = with pkgs; [
          # for precommit hook
          pnpm
          husky
        ];
        icodex-cli = import ./icodex-cli {
          inherit pkgs monorepo-deps;
        };
        icodex-rs = import ./icodex-rs {
          pkgs = pkgsWithRust;
          inherit monorepo-deps;
        };
      in
      rec {
        packages = {
          icodex-cli = icodex-cli.package;
          icodex-rs = icodex-rs.package;
        };

        devShells = {
          icodex-cli = icodex-cli.devShell;
          icodex-rs = icodex-rs.devShell;
        };

        apps = {
          icodex-cli = icodex-cli.app;
          icodex-rs = icodex-rs.app;
        };

        defaultPackage = packages.icodex-cli;
        defaultApp = apps.icodex-cli;
        defaultDevShell = devShells.icodex-cli;
      }
    );
}
