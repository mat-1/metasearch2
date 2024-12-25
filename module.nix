self: {
  config,
  pkgs,
  lib,
  ...
}: {
  options.services.metasearch = {
	  enable = lib.mkEnableOption "metasearch";
  };

  config = {
    systemd.services.metasearch = {
      wantedBy = ["multi-user.target"];
      after = ["network.target"];
      description = "a cute metasearch engine";
      serviceConfig = {
        ExecStart = "${self.packages.${pkgs.system}.default}/bin/metasearch";
        DynamicUser = false;
      };
    };
  };
}
