# Going live

Everything here needs a human with credentials. The kit runs locally without
any of it: `just setup && just db-up && just dev`.

Work through it in order. Each step is independent of the ones below it.

## 1. Replace the fnox age recipient

`fnox.toml` ships a placeholder public key with no matching private key, so
encryption fails until you swap it. Do this before any deploy.

```bash
mkdir -p ~/.config/fnox
age-keygen -o ~/.config/fnox/age.txt
age-keygen -y ~/.config/fnox/age.txt        # prints your public key
```

Paste that public key into `recipients` in `fnox.toml`, then store the one
secret production refuses to start without:

```bash
fnox set RWK_SESSION__SECRET               # any long random string
fnox exec -- just dev-api                  # confirm it round-trips
```

Never commit `~/.config/fnox/age.txt` or a plaintext secret. `.gitignore`
already covers `age.txt` and `FNOX_AGE_KEY`.

## 2. Point mail at a real SMTP server

Mailpit is a local catcher. It accepts everything and delivers nothing, so
production needs real credentials. Any provider works, since the mailer speaks
plain SMTP rather than a vendor SDK.

```bash
fnox set RWK_MAIL__SMTP_PASSWORD
```

Set `RWK_MAIL__FROM`, `RWK_MAIL__SMTP_HOST`, `RWK_MAIL__SMTP_PORT`, and
`RWK_MAIL__SMTP_USERNAME` alongside it. With a username set, the mailer uses a
STARTTLS relay; with none, it assumes a local catcher.

Verification and password-reset links are built from `RWK_APP_URL`, so that
must be the public origin or every link in every email points at localhost.

## 3. Enable OAuth

Both providers stay off until their client id *and* secret are set. An
unconfigured provider returns 404 rather than a broken redirect.

Create the apps with this callback URL, substituting your origin and provider:

```
{RWK_APP_URL}/api/auth/oauth/{google|github}/callback
```

Then set all four values:

```
RWK_OAUTH__GOOGLE_CLIENT_ID       RWK_OAUTH__GOOGLE_CLIENT_SECRET
RWK_OAUTH__GITHUB_CLIENT_ID       RWK_OAUTH__GITHUB_CLIENT_SECRET
```

Confirm it worked by signing in with each provider and checking that a row
appears in `oauth_accounts`. The state handling, PKCE, and account-linking
rules are covered by tests, but no test can talk to a real provider.

Linking rules, in order: an existing `(provider, provider_user_id)` pair wins;
otherwise an existing **verified** email is linked; otherwise a new user is
created with no password and a verified address. An existing *unverified*
email is rejected with 409, which is what stops a stranger from claiming an
account by registering its address at a provider.

## 4. Deploy to a VM

The app is one image that also serves the built frontend, so a VM needs only
Docker and this repo's compose file.

```bash
just deploy user@host
```

That builds for `linux/amd64` by default, because most VMs are x86 while Apple
Silicon builds arm64 natively. Pass a platform for an arm VM:

```bash
just deploy user@host linux/arm64
```

The recipe builds the image, streams it over SSH, copies `docker-compose.prod.yml`
and `fnox.toml`, and starts the stack. Migrations run on startup.

On [exe.dev](https://exe.dev/docs/proxy.md), publish the port and set the
origin to match:

```bash
ssh exe.dev share port <vmname> 8080
ssh exe.dev share set-public <vmname>     # only if it should be public
```

Set `RWK_APP_URL` to the resulting `https://…` origin. It drives OAuth
redirects, email links, the CORS origin, and the origin check on mutating
requests, so a wrong value breaks sign-in in confusing ways.

## 5. Decide what requires a verified email

Registration logs the user in immediately and sends a verification email, and
nothing requires them to act on it. To gate a route, swap the extractor:

```rust
pub async fn create_project(RequireVerified(user): RequireVerified, ...)
```

That returns 403 with the `/problems/email-not-verified` problem type, which
the frontend already distinguishes from an ordinary permission failure. The
authed layout shows a banner with a resend button while an address is
unverified.

## Checks worth running once

```bash
just ci                    # everything: checks, tests, browser flow, image
```

Against a fresh clone of the template, confirm the rename is clean:

```bash
just init myapp
grep -r rwk .              # should find nothing
```
