{
  description = "a tool for remembering and using commands";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs = { self, nixpkgs }:
    let
      systems = [ "x86_64-linux" "aarch64-linux" ];
      forAllSystems = nixpkgs.lib.genAttrs systems;

      spellbookModule = moduleType: { config, lib, pkgs, ... }:
        let
          cfg = config.programs.spellbook;
          spellbookPkg = self.packages.${pkgs.stdenv.hostPlatform.system}.default;
        in
        {
          options.programs.spellbook = {
            enable = lib.mkEnableOption "spellbook TUI command manager";

            package = lib.mkOption {
              type = lib.types.package;
              default = spellbookPkg;
              defaultText = lib.literalExpression "spellbook.packages.\${pkgs.stdenv.hostPlatform.system}.default";
              description = "The spellbook package to use.";
            };
          };

          config = lib.mkIf cfg.enable (
            if moduleType == "nixos" then {
              environment.systemPackages = [ cfg.package ];
            } else {
              home.packages = [ cfg.package ];
            }
          );
        };
    in
    {
      packages = forAllSystems (system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
        in {
          default = pkgs.callPackage ./package.nix { };
        });

      devShells = forAllSystems (system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
        in {
          default = pkgs.mkShell {
            packages = with pkgs; [
              cargo
              rustc
              rustfmt
              gcc
            ];
          };
        });

      nixosModules.default = spellbookModule "nixos";
      homeManagerModules.default = spellbookModule "home-manager";
    };
}
