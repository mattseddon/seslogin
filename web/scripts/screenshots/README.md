# Screenshot scripts

Playwright scripts used to capture before/after screenshots of key routes
(originally written for the Tailwind CSS migration on `sdunster.tailwind`).
Also useful as a starting point for automated end-to-end checks.

## Setup

```
cd web
npm install --no-save playwright
npx playwright install chromium
npm run dev   # vite dev server on :5173
```

Each script also needs the API server running on :8000. Since these scripts
authenticate by seeding a token into `localStorage` rather than performing a
real login, the API must be started with one of the dev-only auth-bypass
flags (see `api/src/bin/poem.rs`) so the seeded token is accepted without a
real session/user existing:

```
# for capture-admin.mjs (acts as a super-user)
cd api && RUST_LOG=info cargo run --bin poem -- --dev-auth-user <user-id>

# for capture-kiosk.mjs (acts as a kiosk session)
cd api && RUST_LOG=info cargo run --bin poem -- --dev-auth-session <session-id>
```

`--dev-auth-*` bypasses token verification only — it does not enable
mutations. Run without `--enable-mutations` when just taking screenshots so
nothing gets written.

**Never use these flags, or point `DB_PREFIX` at real data, against anything
other than a local/throwaway environment.**

## Scripts

- `capture-public.mjs` — unauthenticated routes: home, admin login, kiosk setup.
- `capture-admin.mjs` — admin routes. Requires `--dev-auth-user` and a
  location named "Test Unit" (or edit the `LOCATION_NAME` constant) so no
  real unit data appears in screenshots.
- `capture-kiosk.mjs` — kiosk scan flow. Requires `--dev-auth-session`.

## Usage

```
node scripts/screenshots/capture-public.mjs <output-dir>
node scripts/screenshots/capture-admin.mjs <output-dir>
node scripts/screenshots/capture-kiosk.mjs <output-dir>
```
