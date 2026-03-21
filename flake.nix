{
  description = "A terminal-based spellbook manager";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs }:
    let
      overlay = final: prev: {
        rustc = prev.rustc;
      };
      pkgs = import nixpkgs {
        overlays = [ overlay ];
        system = "x86_64-linux";
      };
    in
    {
      devShells.x86_64-linux.default = pkgs.mkShell {
        buildInputs = with pkgs; [
          rustc
          cargo
        ];

        RUST_BACKTRACE = "1";
      };
    };
}
