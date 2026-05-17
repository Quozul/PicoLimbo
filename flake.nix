{
  description = "Lightweight Minecraft limbo server";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs =
    { self, nixpkgs, ... }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "aarch64-darwin"
      ];
      forAllSystems = nixpkgs.lib.genAttrs systems;
    in
    {
      overlays.default = final: _prev: {
        picolimbo = final.callPackage ./nix/package.nix { };
      };

      packages = forAllSystems (
        system:
        let
          pkgs = nixpkgs.legacyPackages.${system}.extend self.overlays.default;
        in
        {
          default = pkgs.picolimbo;
        }
      );

      nixosModules.default =
        {
          pkgs,
          config,
          lib,
          ...
        }:
        {
          imports = [ ./nix/module.nix ];
          config = lib.mkIf config.services.picolimbo.enable {
            nixpkgs.overlays = [ self.overlays.default ];
            services.picolimbo.package = lib.mkDefault pkgs.picolimbo;
          };
        };
    };
}
