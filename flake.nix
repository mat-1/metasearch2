{
  description = "a cute metasearch engine";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    crane.url = "github:ipetkov/crane";

    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };
  };

  outputs = inputs @ {
    self,
    crane,
    flake-parts,
    ...
  }:
    flake-parts.lib.mkFlake {inherit inputs;} {
      systems = ["x86_64-linux" "x86_64-darwin" "aarch64-darwin" "aarch64-linux"];
      flake.nixosModules.default = import ./module.nix self;

      perSystem = {
        pkgs,
        system,
        ...
      }: let
        craneLib = crane.mkLib pkgs;

        assetFilter = path: _type: (pkgs.lib.strings.hasPrefix (toString ./src/web/assets) path);
        sourceFilter = path: type: (craneLib.filterCargoSources path type) || (assetFilter path type);

        # Common arguments can be set here to avoid repeating them later
        # Note: changes here will rebuild all dependency crates
        commonArgs = {
          src = pkgs.lib.cleanSourceWith {
            src = ./.;
            filter = sourceFilter;
            name = "source"; # Be reproducible, regardless of the directory name
          };
          strictDeps = true;

          buildInputs = [
            # Add additional build inputs here
          ];
        };

        metasearch2 = craneLib.buildPackage (commonArgs
          // {
            cargoArtifacts = craneLib.buildDepsOnly commonArgs;

            # Additional environment variables or build phases/hooks can be set
            # here *without* rebuilding all dependency crates
            # MY_CUSTOM_VAR = "some value";
          });
      in {
        formatter = pkgs.alejandra;

        checks = {
          inherit metasearch2;
        };

        packages.default = metasearch2;

        apps.default = {
          type = "app";
          program = "${self.packages.${system}.default}/bin/metasearch";
        };

        devShells.default = craneLib.devShell {
          checks = self.checks.${system};

          # Additional dev-shell environment variables can be set directly
          # MY_CUSTOM_DEVELOPMENT_VAR = "something else";

          # Extra inputs can be added here; cargo and rustc are provided by default.
          packages = [
            # pkgs.ripgrep
          ];
        };
      };
    };
}
