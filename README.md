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

This checks formatting, linting, types, and the production build.

## Production

```sh
pnpm build
pnpm start
```

The static server listens on `0.0.0.0:4173` by default. Set `HOST` or `PORT` to override either value. `GET /health` returns the deployment health status.

## Desktop downloads

The server resolves Windows installers from `ducnguyen67201/TroApp` GitHub
Releases. Mac downloads continue to use `ducnguyen67201/asdfaefqwefq`.
Publish the verified, signed Windows installer as a GitHub Release asset; Actions
artifacts alone are temporary and cannot serve as public downloads. Windows
prereleases are labelled previews, while legacy Mac prereleases remain labelled
unsigned previews. Stable releases take priority over previews for each platform.
The resolver caches metadata for five minutes and only accepts asset URLs from the
owning repository. Windows never falls back to the legacy repository.
