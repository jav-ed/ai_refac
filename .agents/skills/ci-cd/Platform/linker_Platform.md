# Platform

Forgejo and Woodpecker platform layer. Use this when checking or repairing the shared CI/CD services before looking at one concrete application repo.

## Routing

- [Service checks](Checks/service_Checks.md): Forgejo reachability, Woodpecker server and agent status, dashboard reachability, local backend readiness, and runtime repo handoff.
- [Access checks](Checks/access_Checks.md): current operator and machine access to Forgejo, Woodpecker, git remotes, and secret-backed API checks.
- [API discovery](API_Work/api_Discovery.md): version, Swagger, configured forge, and runtime feature hints exposed by Forgejo/Woodpecker.
- [Secrets](Secrets/linker_Secrets.md): expected Woodpecker/Forgejo secret names and the boundary between service runtime secrets and API repair tokens.
- [Secrets and access](Secrets/secrets_And_Access.md): how to use Fnox for platform checks without leaking decrypted values.

Application-specific `.woodpecker.yml`, repo webhooks, deploy targets, and daily deploy flow live under [Application](../Application/linker_Application.md).
