{
  config,
  pkgs,
  lib,
  ...
}:

let
  inherit (lib)
    literalExpression
    mkDefault
    mkEnableOption
    mkIf
    mkOption
    mkPackageOption
    types
    ;
  cfg = config.services.picolimbo;
  settingsFormat = pkgs.formats.toml { };
  configFile = settingsFormat.generate "server.toml" cfg.settings;
  listenPort = lib.toInt (lib.last (lib.splitString ":" cfg.settings.bind));
in
{
  options.services.picolimbo = {
    enable = mkEnableOption "PicoLimbo, a lightweight Minecraft limbo server written in Rust";

    package = mkPackageOption pkgs "picolimbo" { };

    openFirewall = mkOption {
      type = types.bool;
      default = false;
      example = true;
      description = "Open the firewall port derived from {option}`services.picolimbo.settings.bind`.";
    };

    settings = mkOption {
      type = settingsFormat.type;
      default = { };
      description = ''
        PicoLimbo configuration as a Nix attribute set, serialised to
        {file}`/etc/picolimbo/server.toml` at activation time.

        Environment variable placeholders (`''${VAR}`) are expanded by the server at start-up.

        Full reference: https://picolimbo.quozul.dev/config/introduction.html
      '';
      example = literalExpression ''
        {
          bind = "0.0.0.0:25565";
          welcome_message = "<green>You are in limbo!</green>";
          default_game_mode = "spectator";

          forwarding = {
            method = "MODERN";
            secret = "''${VELOCITY_SECRET}";
          };

          server_list.message_of_the_day = "<gold>My Server</gold>";

          tab_list = {
            enabled = true;
            header = "<bold>Limbo</bold>";
            footer = "<green>Reconnecting soon…</green>";
          };
        }
      '';
    };
  };

  config = mkIf cfg.enable {
    services.picolimbo.settings = {
      # Provide a default bind address so the generated TOML is never empty.
      # An empty file causes the server to attempt a write back to the path,
      # which fails against the read-only /etc location.
      bind = mkDefault "0.0.0.0:25565";
    };

    assertions = [
      {
        assertion =
          (cfg.settings.forwarding or { }).method or "NONE" != "MODERN"
          || (cfg.settings.forwarding or { }) ? secret;
        message = "services.picolimbo.settings.forwarding.secret must be set when method is \"MODERN\"";
      }
      {
        assertion =
          (cfg.settings.forwarding or { }).method or "NONE" != "BUNGEE_GUARD"
          || (cfg.settings.forwarding or { }).tokens or [ ] != [ ];
        message = "services.picolimbo.settings.forwarding.tokens must be non-empty when method is \"BUNGEE_GUARD\"";
      }
    ];

    environment.etc."picolimbo/server.toml".source = configFile;

    systemd.services.picolimbo = {
      description = "PicoLimbo, a lightweight Minecraft limbo server written in Rust";
      wantedBy = [ "multi-user.target" ];
      after = [ "network.target" ];

      serviceConfig = {
        ExecStart = "${lib.getExe cfg.package} --config /etc/picolimbo/server.toml";
        StateDirectory = "picolimbo";
        WorkingDirectory = "/var/lib/picolimbo";
        DynamicUser = true;
        Restart = "always";
      };
    };

    networking.firewall.allowedTCPPorts = mkIf cfg.openFirewall [ listenPort ];
  };
}
