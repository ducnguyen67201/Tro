# Tro marketing site

This repository contains only Tro's bilingual marketing page. It is a React 19 and Vite application served by a small static Node.js server in production.

## Development

Requirements: Node.js 22+ and pnpm 10.23.

```sh
pnpm install
pnpm dev
```

The development site runs at `http://127.0.0.1:4173`.

## Verification

```sh
pnpm check
```

This checks formatting, linting, types, download resolver tests, and the production build.

## Production

```sh
pnpm build
pnpm start
```

The static server listens on `0.0.0.0:4173` by default. Set `HOST` or `PORT` to override either value. `GET /health` returns the deployment health status.

## Desktop downloads

The server resolves signed Mac and Windows installers from
`ducnguyen67201/TroApp` GitHub Releases. It selects each platform's newest stable
release independently, falling back to a preview only when that platform has no
stable installer. A Mac-only release therefore keeps the existing Windows
installer available. Legacy repositories are never download fallbacks.

Publish the exact verified files as public GitHub Release assets; temporary
Actions artifacts and draft releases are not public download sources. Platform
release tags such as `macos-v0.1.0-59ccc66` leave the Windows `v0.1.0` tag intact.
The displayed app version comes from the canonical installer filename, not the
release tag. Mac buttons prefer DMG and fall back to ZIP; each button displays its
actual format, size and app version. An architecture without a published asset
stays unavailable until its verified installer is added.

The resolver caches metadata for five minutes, shares one paginated GitHub release
history across platforms, and accepts only exact asset URLs from the owning
TroApp release. It scans at most 1,000 releases with a 10-second request timeout;
request failures retain a previously cached successful result. Downloads are
redirected through `/downloads/latest/macos-arm64`,
`/downloads/latest/macos-x64`, and `/downloads/latest/windows-x64`.
