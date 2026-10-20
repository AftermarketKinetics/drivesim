# drivesim-broker: the only drivesim container given the Docker socket.
# Stdlib Python, the docker CLI and the broker script, nothing else.
#   nix-build deploy/broker-image.nix && docker load < result
#
# Runs as root inside the container: holding the socket is root-equivalent on
# the host anyway, and the socket's group id differs between hosts. What keeps
# it safe is the broker's single fixed command (broker/broker.py.md) and that
# compose exposes it only on the internal network the server shares. Adapted
# from the analyzer's deploy/broker-image.nix (2026-10-10).
{ pkgs ? import ../nix/pkgs.nix { } }:

pkgs.dockerTools.buildLayeredImage {
  name = "drivesim-broker";
  tag = "latest";
  contents = [ pkgs.python3 pkgs.docker-client pkgs.coreutils pkgs.dockerTools.fakeNss ];
  extraCommands = ''
    mkdir -p app tmp && chmod 1777 tmp
    cp ${../broker/broker.py} app/broker.py
  '';
  config = {
    Entrypoint = [ "${pkgs.python3}/bin/python3" "/app/broker.py" ];
    Env = [ "DRIVESIM_BROKER_BIND=0.0.0.0:8711" "PATH=/bin" ];
  };
}
