# broker-image.nix

## Function
Builds `drivesim-broker:latest`: the one drivesim container with the
Docker socket, which starts an import sandbox per upload.

## Interface
- `nix-build deploy/broker-image.nix && docker load < result` (or
  `deploy/push.sh`).
- Listens on `0.0.0.0:8711` (`DRIVESIM_BROKER_BIND`), only on compose's
  internal `drivesim` network.
- Compose sets `DRIVESIM_ROBOTS_HOST_ROOT` (a host path) and
  `DRIVESIM_SANDBOX_RUNTIME=runsc`, and mounts the socket.

## Implementation
Copied from the analyzer's `deploy/broker-image.nix`: stdlib Python, the
docker CLI, coreutils and `broker/broker.py`. It runs as root, because the
socket is root-equivalent anyway and its group id varies by host. Safety
comes from the broker's single fixed command and the internal network.

## Assertions
- [ ] Only `broker/broker.py` and its runtime are in the image.
- [ ] Never on a network the public can reach.
