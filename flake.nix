{
  description = "PhotoSort";

  inputs = {
    flake-parts.url = "github:hercules-ci/flake-parts";
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    naersk.url = "github:nix-community/naersk";
    naersk.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = inputs@{ flake-parts, naersk, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [ "x86_64-linux" "aarch64-linux" "aarch64-darwin" "x86_64-darwin" ];
      perSystem = { config, self', inputs', pkgs, lib, system, ... }:
        let
          naersk' = pkgs.callPackage naersk { };
        in
        rec {
          packages.default = naersk'.buildPackage {
            src = ./.;
            gitAllRefs = true;
            meta.mainProgram = "timetrax";
            postInstall = ''
                mkdir -p $out/share/bash-completion/completions
                mkdir -p $out/share/zsh/site-functions
                mkdir -p $out/share/fish/vendor_completions.d

                $out/bin/timetrax completion --shell bash > $out/share/bash-completion/completions/timetrax
                $out/bin/timetrax completion --shell zsh > $out/share/zsh/site-functions/_timetrax
                $out/bin/timetrax completion --shell fish > $out/share/fish/vendor_completions.d/timetrax.fish
            '';
          };
        };
    };
}
