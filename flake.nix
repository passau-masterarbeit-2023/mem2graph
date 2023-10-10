# flake.nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = {nixpkgs, ...}: let
    system = "x86_64-linux";
    #       ↑ Swap it for your system if needed
    #       "aarch64-linux" / "x86_64-darwin" / "aarch64-darwin"
    pythonPackages = pkgs.python311Packages;
    pkgs = nixpkgs.legacyPackages.${system};
  in {
    devShells.${system}.default = pkgs.mkShell {
      packages = [
        pkgs.cargo
        pkgs.rustc
        pkgs.clippy

        pkgs.rust-analyzer
        pkgs.rustup
        pkgs.rustfmt

        pkgs.graphviz
      ];

      # package needed at build and runtime.
      buildInputs = with pkgs; [
        # the environment.
        pythonPackages.python

        # python packages
        pythonPackages.tqdm
      ];

      RUST_BACKTRACE = "1";
      RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
    };
  };
}
