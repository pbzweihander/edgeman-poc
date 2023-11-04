# edgeman

_edgeman_ is an edge device manager.

This is a proof of concept.

## Usage

```
podman system service -t 0

RUST_LOG=info \
  ID=sample-edge1 \
  EDGEMAN_URL=https://example.com \
  FETCH_URL=file://$PWD/sample/sample.yaml \
  FETCH_SCHEDULE="1/10 * * * * *" \
  cargo run --bin edge
```

## Current problems

- Cannot replace pod
  - Reconcile always fails with `pod already exists` error
  - https://github.com/containers/podman/commit/91df369ae6807d5d3c0adf37ea5caeb883c0284e must be released
  - Or we should manually compare new pod spec and existing pod
- Lack of pod versioning
  - Should detect currently deployed pod's timestamp and compare with fetched spec
- Lack of pod healthchecking

## TODOs

- Spec signing
  - Should prevent malicious attacker replacing pod specs
  - libsodium would useful
- Health signing
  - Should sign individual edge health pushing
  - libsodium would useful
- edgeman server implementation
  - Receive edge healths and store somewhere
  - Edge health visualizer (Web UI?)

## License

[MIT License](./LICENSE)
