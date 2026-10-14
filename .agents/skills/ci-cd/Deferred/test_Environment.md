# Optional Internal Test Environment

This file is WIP design notes, not active setup instructions. Use it only when the user explicitly wants to design or implement the optional `test` branch deployment path.

The current mandatory CI/CD path is `main` to production. The optional `test` branch may become useful for safe non-production inspection: experiments, client corrections, internal QA, or a final production-candidate check before public launch.

There are two different cases:

- Private/owned projects: if we own and control the domain, a pattern like `test.javedab.com` can be acceptable when intentionally configured.
- Client projects: never use the client's public domain for non-production inspection. Use a Zetun-owned hostname and NetBird-only access instead.

## Contents

- [Proposed rule](#proposed-rule)
- [Access requirement](#access-requirement)
- [Possible server layout](#possible-server-layout)
- [Required discipline](#discipline-needed)

## Proposed Rule

For client projects, if implemented, the mapping should be:

```text
test branch -> Zetun-owned internal hostname -> NetBird-only access
main branch -> real client production domain -> public access
```

Never use these for client projects:

```text
test.<client-domain>
preview.<client-domain>
staging.<client-domain>
any hostname under the client's public domain
```

Use a Zetun-controlled hostname instead. Possible naming ideas:

```text
client-1042.zetunweb.com
p1042.zetunweb.com
p1042-mueller.zetunweb.com
```

Prefer hyphens, not underscores. If client-identifying names are sensitive, prefer the shorter project id form, such as `p1042.zetunweb.com`.

For private/owned projects, document the exception explicitly in the project docs before using an owned-domain test hostname:

```text
test branch -> test.<owned-domain> -> access policy chosen for that owned project
main branch -> <owned-domain> -> production
```

## Access Requirement

The internal hostname must be reachable only from NetBird-connected devices. Do not treat DNS naming alone as access control.

Before this becomes active setup, design and test:

- Caddy access control for NetBird-only clients.
- DNS or internal resolver behavior for the Zetun-owned hostname.
- Server directory layout for `test` and `prod`/`main` targets.
- Deploy script branch mapping and overwrite guards.
- Clear rollback and verification commands for the internal target.

## Possible Server Layout

This is a proposal, not the current standard:

```text
/srv/web/clients/p1042-mueller/
├── test/
│   ├── releases/
│   └── current
└── prod/
    ├── releases/
    └── current
```

Possible mapping:

```text
test branch -> /srv/web/clients/p1042-mueller/test/dist -> p1042.zetunweb.com -> NetBird-only
main branch -> /srv/web/clients/p1042-mueller/prod/dist -> client public domain -> public
```

The current template uses `main/dist` for the active production path. Do not silently switch to `prod/dist`; that is a separate migration decision.

## Discipline Needed

The `test` branch can mean both experimental work and final pre-launch inspection. That flexibility is useful, but it needs discipline:

```text
test may be messy during development.
test must be clean before final approval.
main must only receive approved production work.
```

Before launch, the future workflow could be:

```text
build on test
inspect through the NetBird-only Zetun hostname
fix issues on test
approve
merge test -> main
deploy main to the public client domain
```

Keep this deferred until the Caddy, NetBird, DNS, deploy-script, and folder-structure decisions are proven end-to-end.
