# StellarEscrow Documentation

> Central hub for all infrastructure, operational, and architectural documentation.

## Guides

| Document | Description |
|----------|-------------|
| [Architecture](architecture.md) | System design, component inventory, CLI flow, security model, data flow |
| [Deployment](deployment.md) | Environment setup, CI/CD pipeline, production deployment, rollback |
| [Operations](operations.md) | Daily health checks, backups, secrets rotation, scaling, change management |
| [Troubleshooting Runbook](runbooks/troubleshooting.md) | Problem → Diagnosis → Resolution for all common incidents |

## Quick Links

| Need | Go To |
|------|-------|
| First time deploying | [Deployment — Docker Stack Deployment](deployment.md#docker-stack-deployment) |
| Something is broken right now | [Troubleshooting — First-Responder Checklist](runbooks/troubleshooting.md#first-responder-checklist) |
| Rotating API keys | [Operations — Secrets Management](operations.md#secrets-management) |
| Understanding the system | [Architecture](architecture.md) |
| Restoring from backup | [Troubleshooting — RB-008](runbooks/troubleshooting.md#rb-008-database-restore-from-backup) |
| Full host rebuild | [Troubleshooting — RB-009](runbooks/troubleshooting.md#rb-009-complete-host-rebuild) |
| CI/CD pipeline | [Deployment — CI/CD Pipeline](deployment.md#cicd-pipeline) |
| Scaling the platform | [Operations — Scaling](operations.md#scaling) |

## Additional References

| Document | Location |
|----------|----------|
| Analytics API | [docs/analytics.md](analytics.md) |
| Security policies | [docs/security.md](security.md) |
| Environment variables | [docs/environments.md](environments.md) |
| Bridge integration | [docs/BRIDGE_INTEGRATION.md](BRIDGE_INTEGRATION.md) |
| Testing guidelines | [docs/testing/TESTING_GUIDELINES.md](testing/TESTING_GUIDELINES.md) |
| Test strategy | [docs/testing/TEST_STRATEGY.md](testing/TEST_STRATEGY.md) |
| Disaster recovery | [DISASTER_RECOVERY.md](../DISASTER_RECOVERY.md) |
| Performance | [PERFORMANCE.md](../PERFORMANCE.md) |
