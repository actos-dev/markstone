# markstone — Release Roadmap

Every binding in `bindings/` is built and tested by CI, but only two
registries receive releases today. This file records what the remaining ones
need, so that adding a channel is a matter of doing the listed work rather
than rediscovering it.

The release workflow before this split, with draft jobs for every registry,
is `.github/workflows/release.yml` at commit `80c781c`. Those jobs were never
run, so treat them as a starting point, not as working code.

## Released

| Registry | Packages | Workflow job |
|---|---|---|
| crates.io | `markstone-core`, `markstone-actos`, `markstone` | `publish-crates` |
| npm | `markstone` (eight native addons + WebAssembly) | `publish-npm` |

A release is a `v*` tag whose version matches every manifest
(`scripts/check_versions.py`). Running the Release workflow by hand is the
dry run: it builds and tests everything and publishes nothing.

## Planned

### PyPI — `markstone`

- An abi3 wheel per platform (the same eight as npm) plus an sdist, built
  with maturin. Linux wheels must be manylinux/musllinux tagged.
- Account: create the PyPI project and configure **trusted publishing** for
  `actos-dev/markstone`, which removes the need for a token secret.
- Test each wheel on its own platform before upload, the way `node-addon`
  tests each addon.

### Go module — `github.com/actos-dev/markstone/bindings/go`

- Go modules are published by tagging, but the tag for a module in a
  subdirectory is `bindings/go/vX.Y.Z`, not `vX.Y.Z`.
- The module embeds the C ABI library (`internal/embeds`). Prebuilt
  `markstone-abi` libraries for every platform must be committed, or
  generated and committed by the release, before that tag is pushed, because
  the Go proxy serves the tagged tree as-is.
- Decide whether shipping eight binaries inside the module is acceptable, or
  whether the module downloads a verified binary from the GitHub release on
  first use instead.

### Maven Central — `markstone`

- The group id is `io.github.dethrandir`, tied to a personal account. Choose
  the final namespace (for example a verified domain) before the first
  release, because a published coordinate can never move.
- Needs a Sonatype Central Portal account, a verified namespace, a GPG key
  and the secrets `MAVEN_CENTRAL_USERNAME`, `MAVEN_CENTRAL_PASSWORD`,
  `MAVEN_GPG_PRIVATE_KEY`, `MAVEN_GPG_PASSPHRASE`.
- The JAR bundles the C ABI library for every platform
  (`scripts/pack_natives.py`).

### NuGet — `Markstone`

- Needs a nuget.org account and `NUGET_API_KEY`, or trusted publishing.
- The package carries the C ABI library under `runtimes/{rid}/native/`.

### C ABI archives on GitHub releases

- Go, the JVM and .NET all consume the `markstone-abi` shared library. Once
  any of them ships, the release should also build that library for all eight
  platforms, attach the archives and a `SHA256SUMS` file to the GitHub
  release, and feed the same binaries to those packages.
