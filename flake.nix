{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = {
    self,
    nixpkgs,
  }: let
    system = "x86_64-linux";
    pkgs = nixpkgs.legacyPackages.${system};

    runtimeLibs = with pkgs; [
      openssl
      stdenv.cc.cc.lib
    ];
  in {
    devShells.${system}.default = pkgs.mkShell {
      nativeBuildInputs = with pkgs; [
        pkg-config
        rustup
        git
      ];

      buildInputs = runtimeLibs;

      LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath runtimeLibs;

      shellHook = ''
        export CARGO_INSTALL_ROOT=$PWD/.nix-cargo
        export PATH=$CARGO_INSTALL_ROOT/bin:$PATH
      '';
    };
  };
}
