{ omglol-ddns }:
{ config, lib, pkgs, ... }:

with lib;

let
  cfg = config.services.omglol-ddns;
  pkg = omglol-ddns;
in {
  options = {
    services.omglol-ddns = {

      enable = mkEnableOption (lib.mdDoc "omglol-ddyns");

      interval = mkOption {
        type = types.str;
        default = "15min";
        description = lib.mdDoc "How often to update the entries";
      };

      name = mkOption {
        type = types.str;
        description =
          lib.mdDoc "The name of you omg.lol account. e.g., name.omg.lol";
      };

      subdomain = mkOption {
        type = types.str;
        description = lib.mdDoc
          "The subdomain you want to create/update. e.g., subdomain.yourname.omg.lol";
      };

      apiKey = mkOption {
        type = types.str;
        description = lib.mdDoc "Omg.lol API key";
      };

      environmentFile = mkOption {
        type = types.str;
        description = lib.mdDoc ''
          File containing the OMGLOL_API_KEY in the format of an EnvironmentFile as described by systemd.exec(5)
        '';
      };
    };
  };

  config = mkIf cfg.enable {

    systemd.timers.omglol-ddns = {
      description = "omglol-ddns timer";
      wantedBy = [ "timers.target" ];
      timerConfig = {
        OnBootSec = cfg.interval;
        OnUnitActiveSec = cfg.interval;
      };
    };

    systemd.services.omglol-ddns = {
      description = "omglol-ddns service";
      serviceConfig = {
        ExecStart =
          "${pkg}/bin/omglol-ddns--name ${cfg.name} --subdomain ${cfg.subdomain}"
          + (lib.optionalString (cfg.apiKey != null)
            " --api-key ${cfg.apiKey}");
        EnvironmentFile = "${cfg.environmentFile}";
        DynamicUser = true;
      };
    };

  };
}
