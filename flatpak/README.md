# Flatpak / Flathub packaging

Files here support distributing GitStream via [Flathub](https://flathub.org/).
Full plan: [`docs/superpowers/plans/2026-05-17-flathub-publishing.md`](../docs/superpowers/plans/2026-05-17-flathub-publishing.md).

| File | Purpose |
|---|---|
| `io.github.avv7bc.gitstream.yml` | Flatpak manifest (**skeleton** — see open items inside) |
| `git.json` | bundled `git` build module (from kernel.org, mirrors git-cola) |
| `io.github.avv7bc.gitstream.metainfo.xml` | AppStream metadata (needs real screenshots) |
| `io.github.avv7bc.gitstream.desktop` | Desktop entry |
| `io.github.avv7bc.gitstream.png` | 512×512 app icon |

## Generate offline sources (Stage 2 — required before it builds)

Flathub builds without network access; npm and cargo dependencies must be vendored:

```bash
# from https://github.com/flatpak/flatpak-builder-tools
flatpak-node-generator npm ../package-lock.json -o generated-sources-node.json
python3 flatpak-cargo-generator.py ../src-tauri/Cargo.lock -o generated-sources-cargo.json
```

Then uncomment the two `generated-sources-*.json` entries in the manifest `sources`.

## Build & test locally (Stage 4)

```bash
flatpak install flathub org.gnome.Platform//46 org.gnome.Sdk//46 \
  org.freedesktop.Sdk.Extension.rust-stable//24.08 \
  org.freedesktop.Sdk.Extension.node20//24.08

flatpak run org.flatpak.Builder --force-clean --sandbox --user \
  --install --repo=repo builddir io.github.avv7bc.gitstream.yml

flatpak run --command=flatpak-builder-lint org.flatpak.Builder \
  manifest io.github.avv7bc.gitstream.yml
flatpak run --command=flatpak-builder-lint org.flatpak.Builder repo repo

flatpak run io.github.avv7bc.gitstream
```

The linter must pass with no errors before opening the Flathub PR
(against the `new-pr` base branch of `flathub/flathub`).
