{
  nixConfig.bash-prompt-prefix = "(tangent) ";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    systems.url = "github:nix-systems/default";

    naersk = {
      url = "github:nix-community/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    gitignore = {
      url = "github:hercules-ci/gitignore.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs @ {self, ...}: let
    forAllSystems = f:
      inputs.nixpkgs.lib.genAttrs
      (import inputs.systems)
      (system: let
        pkgs = import inputs.nixpkgs {
          inherit system;
          overlays = [inputs.rust-overlay.overlays.default];
        };
      in
        f pkgs.stdenv.hostPlatform.system pkgs);

    meta = (inputs.nixpkgs.lib.importTOML ./tangent/Cargo.toml).package;

    workspace = inputs.nixpkgs.lib.cleanSourceWith {
      name = "${meta.name}-${meta.version}-src";
      src = ./.;
      filter = inputs.gitignore.lib.gitignoreFilterWith {
        basePath = ./.;
        extraRules =
          # gitignore
          ''
            flake.*
            LICENSE.md
            README.md
            .github
            makefile
            *.sf2
            *.mid
            docs
          '';
      };
    };
  in {
    packages = forAllSystems (system: pkgs: let
      rustToolchain =
        pkgs.rust-bin.nightly.latest.default.override
        {extensions = ["rust-src" "rust-analyzer"];};

      naersk = pkgs.callPackage inputs.naersk {
        rustc = rustToolchain;
        cargo = rustToolchain;
        clippy = rustToolchain;
      };
    in {
      # `nix run github:PolyMeilex/Tangent`
      default = inputs.self.outputs.packages.${system}.tangent;
      tangent = naersk.buildPackage {
        nativeBuildInputs = [
          pkgs.pkg-config
        ];
        buildInputs = [
          pkgs.alsa-lib
        ];
        meta.mainProgram = "tangent";
        src = workspace;
      };

      # `nix run github:PolyMeilex/Tangent#cli`
      cli = naersk.buildPackage {
        cargoBuildOptions = prev: prev ++ ["--package" "tangent-cli"];
        nativeBuildInputs = [
          pkgs.pkg-config
          pkgs.ffmpeg_8.dev
        ];
        buildInputs = [pkgs.ffmpeg_8.lib];
        meta.mainProgram = "tangent-cli";
        src = workspace;
      };
    });

    devShells = forAllSystems (system: pkgs: let
      rustToolchain =
        pkgs.rust-bin.nightly.latest.default.override
        {extensions = ["rust-src" "rust-analyzer"];};

      runtimeLibs = with pkgs; [
        wayland
        libxkbcommon
        libGL
        vulkan-loader
        alsa-lib
        libx11
        libxcursor
        libxrandr
        libxi
        ffmpeg_8.lib
      ];
    in {
      # `nix develop github:PolyMeilex/Tangent`
      default = pkgs.mkShell {
        nativeBuildInputs = with pkgs; [
          rustToolchain
          pkg-config
          ffmpeg_8.dev
        ];
        buildInputs = with pkgs; [
          alsa-lib
        ] ++ runtimeLibs;
        shellHook = ''
          export PKG_CONFIG_PATH="${pkgs.alsa-lib.dev}/lib/pkgconfig:${pkgs.ffmpeg_8.dev}/lib/pkgconfig:$PKG_CONFIG_PATH"
          export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath runtimeLibs}:$LD_LIBRARY_PATH"
        '';
      };
    });
  };
}
