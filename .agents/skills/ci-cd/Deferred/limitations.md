# Limitations

This CI/CD skill is the current project-pipeline workflow for Zetun Astro repos. Treat it as authoritative for adding, checking, and repairing project pipelines, while keeping broader Woodpecker runtime maintenance, source references, service debugging, and project history in `Project_Manag/Docs`.

## Current Scope

- One local Woodpecker server.
- One local Woodpecker agent.
- Local backend, no containers.
- One primary agent machine with absolute project paths.
- Static Astro builds deployed to a Caddy-served Linux host by rsync and atomic symlink swap.
- Project pipelines use the local `Code/CI_CD/` folder shape.
- Secrets are handled through the skill-owned `Platform/Secrets/fnox.toml`.

## Accepted Constraints For Now

- The setup is not yet generalized for multiple developer machines.
- The setup is not yet generalized for multiple Woodpecker agents or worker pools.
- The deploy scripts assume the agent can see the project checkout at a stable absolute path.
- Runtime service secrets and API repair secrets are intentionally kept together in the skill-owned Fnox file; the CI_CD repo root may expose it through a symlink for service convenience.
- The docs may still change as more projects are integrated and the `justfile` conventions become clearer.

## When To Revisit

Rework this area before any of these become true:

- More than one machine should be able to run the same project pipeline.
- More than one Woodpecker agent should be eligible for the same job.
- The project checkout path differs between workers.
- The deploy target needs environment-specific secrets or per-worker credentials.
- The workflow moves from local backend to containers.
- The optional internal `test` environment becomes active instead of WIP design.

## Future Direction

Keep durable, tested project-pipeline instructions here. Astro project skills should remain small routers that point CI/CD tasks back to this dedicated skill in the `CI_CD` repo.
