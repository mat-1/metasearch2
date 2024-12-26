self: {
  config,
  pkgs,
  lib,
  ...
}: let
  cfg = config.services.metasearch;
  port =
    if lib.hasAttr "bind" cfg.settings
    then lib.toInt (builtins.elemAt (lib.splitString ":" cfg.settings.bind) 1)
    else 28019;

  metasearchArgs =
    if cfg.settings != {}
    then " " + pkgs.writers.writeTOML "metasearch.toml" cfg.settings
    else "";
in {
  options.services.metasearch = {
    enable = lib.mkEnableOption "metasearch";
    openFirewall = lib.mkOption {
      type = lib.types.bool;
      default = false;
      description = ''
        Open firewall ports used by metasearch.
      '';
    };
    settings = lib.mkOption {
      type = lib.types.attrs;
      default = {};
      description = ''
        Optional metasearch configuration. If not defined, defaults in `src/config.rs` will be used
      '';
      example = {
        bind = "0.0.0.0:4444";
        ui.show_version_info = true;
        urls = {
          replace = {
            "www.reddit.com" = "old.reddit.com";
          };

          weight = {
            "quora.com" = 0.1;
          };
        };
      };
    };
  };

  config = lib.mkIf cfg.enable {
    systemd.services.metasearch = {
      wantedBy = ["multi-user.target"];
      after = ["network.target"];
      description = "a cute metasearch engine";
      serviceConfig = {
        ExecStart = "${self.packages.${pkgs.system}.default}/bin/metasearch" + metasearchArgs;
      };
    };

    networking.firewall = lib.mkIf cfg.openFirewall {
      allowedTCPPorts = [port];
    };
  };
}
