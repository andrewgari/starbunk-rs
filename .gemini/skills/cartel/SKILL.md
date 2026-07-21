---
name: cartel
description: Triages general requests from The Man and orchestrates collaboration between relevant cartel members and sub-skills.
---

# Cartel Orchestration Dispatcher

When **The Man** makes a general request to the cartel, use this skill to triage the goal, summon the relevant coworkers (agents), and route the execution to the appropriate sub-skills.

---

## Triage Matrix

Evaluate the request and map it to the active crew and execution path:

| Request Type | Active Coworkers | Skill / Command |
|---|---|---|
| **New feature or bugfix** | The Face, The Brains, The Inspector, The Artist, The Mechanic, The Critic | [collaborate-sdlc](file:///mnt/data/tank/workspace/starbunk-rs/.gemini/skills/collaborate-sdlc/SKILL.md) |
| **Address review comments** | The Mechanic, The Artist, The Critic | [address-pr-comments](file:///mnt/data/tank/workspace/starbunk-rs/.gemini/skills/address-pr-comments/SKILL.md) |
| **Fix CI/CD errors / Merge conflicts** | The Inspector, The Mechanic, The Artist | [ci-remediation](file:///mnt/data/tank/workspace/starbunk-rs/.gemini/skills/ci-remediation/SKILL.md) |
| **Production health checks** | The Inspector, The Painter, The Consultant | [health-check](file:///mnt/data/tank/workspace/starbunk-rs/.gemini/skills/health-check/SKILL.md) |
| **Kubernetes release & deploy** | The Face, The Brains, The Inspector, The Consultant | [kubernetes-deploy](file:///mnt/data/tank/workspace/starbunk-rs/.gemini/skills/kubernetes-deploy/SKILL.md) |
| **Repository security & code audit** | The Consultant, The Critic, The Painter | [repo-audit](file:///mnt/data/tank/workspace/starbunk-rs/.gemini/skills/repo-audit/SKILL.md) |

---

## Collaboration Protocol

1. **Summon the Crew:** Clearly state which agents (e.g., **The Face**, **The Brains**, etc.) are joining this operation based on the triage matrix.
2. **Setup the Ledger:** Have **The Inspector** outline the current task checklist in `task.md`.
3. **Execute:** Follow the designated sub-skill path.
4. **Handoff:** Deliver the final status update via **The Face** directly to **The Man**.
