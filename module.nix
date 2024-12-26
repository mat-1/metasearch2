self: {
  config,
  pkgs,
  lib,
  ...
}: let
  metasearchSettings = config.services.metasearch.settings;
  metasearchArgs =
    if metasearchSettings != {}
    then " " + pkgs.writers.writeTOML "metasearch.toml" metasearchSettings
    else "";
in {
  options.services.metasearch = {
    enable = lib.mkEnableOption "metasearch";
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

  config = {
    systemd.services.metasearch = {
      wantedBy = ["multi-user.target"];
      after = ["network.target"];
      description = "a cute metasearch engine";
      serviceConfig = {
        ExecStart = "${self.packages.${pkgs.system}.default}/bin/metasearch" + metasearchArgs;
      };
    };
  };
}
