{
  description = "Kensaku: A fetch tool for the terminal with a fractal twist";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let pkgs = nixpkgs.legacyPackages.${system}; in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "kensaku";
          version = "1.0.0";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
        };
      }
    ) // {
      homeManagerModules.default = { config, lib, pkgs, ... }:
        with lib;
        let
          cfg = config.programs.kensaku;
          tomlFormat = pkgs.formats.toml {};
        in {
          options.programs.kensaku = {
            enable = mkEnableOption "kensaku system fetch";
            
            settings = mkOption {
              type = tomlFormat.type;
              default = {};
              description = "Settings for kensaku/config.toml";
            };
          };

          config = mkIf cfg.enable {
            home.packages = [ self.packages.${pkgs.system}.default ];
            xdg.configFile."kensaku/config.toml".source =
              tomlFormat.generate "config.toml" cfg.settings;
          };
        };
    };
}
