# CovaBot (Rust Rewrite) — Design Record

> Living document. Status tags: **[DECIDED]** = settled, **[OPEN]** = needs a call, **[IDEA]** = parked, not committed.
> Last updated: 2026-06-09

---

## 0. Goal

Rewrite CovaBot in Rust as a Discord personality bot that reads as a *live user*: remembers people, holds opinions that evolve, responds with the cadence and selectivity of a real person — not an eager assistant that replies to everything or a rate-limited one that ghosts mid-conversation.

---

## 1. Core Organizing Principle  **[DECIDED]**

Separate **what Cova *is*** from **when and how Cova *speaks***.

- **Prompt files** own identity, voice, preferences, conversational nuance. Declarative, version-controlled, editable without recompiling.
- **Rust code** owns orchestration: whether to engage the LLM, which prompt fragments to assemble, how much context to attach, reliability of output.

Avoid leakage in either direction:
- Personality logic in Rust → tuning Cova means a redeploy. Bad.
- Orchestration logic in prompts → the LLM is asked to both *be* Cova and *decide whether to respond* in one muddled blob.

---

## 2. Model Tiers  **[DECIDED]**

Code requests a **capability tier**, not a specific model. Provider config maps tier → actual model, layered on the existing provider priority (Ollama → Anthropic → Gemini → OpenAI). Net result is a **tier × provider matrix**.

- **Low** — relevance gate; LLM fallback for conversation tagging. High-frequency, latency-sensitive, deliberately lean context.
- **Med** — background reasoning, not user-facing, runs at human pace:
  - **Stance evolution + formation** — on salient conversation death, reconcile Cova's stances.
  - **Conversation summarization** — compress a dead salient conversation into a retrievable episodic-memory record.
  - **Memory maintenance (forgetting)** — periodic sweep: merge near-duplicate episodic records, decay/drop old low-salience ones.
- **High** — response generation. Full voice + full context budget. Runs least often.

---

## 3. Decision Pipeline (code side)  **[DECIDED]**

Staged, cheap-to-expensive. Each stage returns `respond` / `abstain` / `defer`.

```
message
  → hard filter            (own msgs, ignored channels — microseconds)
  → conversation tagging   (mechanical-first, low-LLM fallback)
  → pull score             (membership + mention + stance; restraint modulates)
  → [pull below floor?] → abstain
  → relevance gate         (LOW tier, lean context) → yes/no + which convo(s) + reason
  → [no?] → abstain
  → generation             (HIGH tier, full context; seeded with gate's reason)
```

Ordering rule: never spend an expensive step on something a cheap step already settled.

### 3.1 Gate → generation contract  **[DECIDED]**

**Gate's role:** catch false positives pull can't (rhetorical questions, sarcasm, name quoted unrelatedly, redundant responses) and articulate intent to seed generation.

```rust
struct GateResult {
    respond: bool,
    conversation_id: ConversationId,
    reason: String,           // seeds generation prompt
    energy: EnergyLevel,      // QuickJab | Normal | Invested
}
```

`reason` + `energy` flow into the generation prompt as **natural framing, not a status block**. `energy` is what produces human cadence.

---

## 4. Responsiveness Model  **[DECIDED]**

Two axes:

- **Pull** (per message): how much this message warrants a response.
  - Direct mention → max
  - Reply to something Cova just said → very high
  - Topic Cova has a stance on → elevated
  - Random chatter → low
- **Restraint** (the old battery): dominating? channel pace? recency of his last message.

**The rule that makes him feel alive:** restraint *modulates* low-pull messages but **never vetoes high-pull ones.**

### 4.1 Pull combination — MAX, not blend  **[DECIDED]**

The three signal families combine by **max**, not by weighted sum:

```rust
let pull = intrinsic.max(conversational).max(stance);
```

The single strongest reason to respond wins. A weak signal never accumulates with other weak signals to clear the bar.

**Why max (chosen):** matches how a real person responds; trivially debuggable; resists the **responds-to-everything** failure mode.

**Blend (weighted sum) — considered and REJECTED:** drifts back toward chatty-about-everything unless weights are kept very tight.

**Guardrail for the agent:** do **not** switch to a blend/weighted-sum. If more ambient behavior is wanted, raise the `stance` term.

### 4.2 Pull values — affinity + stance interaction  **[DECIDED]**

**Participation affinity** scales conversational pull by Cova's status in a conversation:

| Status   | Meaning                              | Affinity |
|----------|--------------------------------------|----------|
| in       | actively in it, recent               | ~1.0     |
| was-in    | spoke earlier, gone cold             | ~0.5     |
| never    | never spoke in it                    | ~0.15    |

**Stance × membership — SCALED.** Stance pull is computed everywhere, but **scaled down in `never` conversations** so only Cova's *strongest* stances clear the bar there.

### 4.3 Global activity level — the volume knob  **[DECIDED]**

An operator-facing master volume, built as a modifier on the **pull floor**:
```
effective_floor = base_floor + channel_baseline_offset + temporary_dampener(decaying)
```

- **Baseline** — persistent, operator-set resting floor. Per-channel with a global default.
- **Temporary dampener** — decays back to baseline over a set duration.
- **Mute** — hard floor; only owner/admin direct address passes.

**Critical property:** raising the floor suppresses **ambient chime-in only** — direct address still clears it, *except* in full mute.

**Triggers:**
- *Manual* — command sets baseline or fires the temporary dampener.
- *Automatic* — negative feedback fires the temporary dampener.

---

## 5. Engagement State  **[DECIDED]**

First-class Rust struct, per channel.

- Tracks who Cova is talking to, how recently, the active thread.
- Active engagement drops the pull threshold near zero for thread participants, then decays as the thread cools.
- Produces human cadence and feeds the generation prompt.

---

## 6. Conversations — Many-to-Many Membership  **[DECIDED]**

Channels are interleaved. A message is tagged with the set of conversations it applies to, weighted by confidence.

Data shape (PostgreSQL via sqlx):
```sql
CREATE TABLE message_conversation (
    message_id      TEXT NOT NULL,
    conversation_id TEXT NOT NULL,
    weight          FLOAT NOT NULL,
    PRIMARY KEY (message_id, conversation_id)
);
```

- A **conversation** = the set of messages tagged to it.
- **Cova-membership** (in / was-in / never) is derived.
- **Bridging:** a message tagged to two conversations is how a topic migrates.

**Tagging mechanic — mechanical-first, LLM-fallback:**
1. Reply references (free, definitive)
2. Embedding similarity to each live conversation's centroid (cheap via pgvector)
3. Participant overlap + recency (free)
4. Escalate to LOW-tier LLM *only* for ambiguous calls.

### 6.1 Tag scheme — two namespaces  **[DECIDED]**

- **Topical tags** — open vocabulary, LLM-generated, **embedded and fuzzy-matched**. Drive conversation membership and stance matching.
- **Structural tags** — closed vocabulary, defined by us, **exact-matched enums in Rust**:

  ```rust
  enum Addressee { Cova, OtherUser, Room }
  enum Intent { Question, Statement, LowEffort }
  ```

  **From Discord metadata (Rust code, no LLM):**
  - `direct_mention` — Cova was @mentioned
  - `reply_to_cova` — Discord reply pointing at a Cova message.

  **From the tagger (LLM, closed enum):**
  - `addressee`: `Cova | OtherUser | Room`
  - `intent`: `Question | Statement | LowEffort`

### 6.2 Tag specificity — broad vs specific  **[DECIDED]**

- **Broad tags = continuity.** Membership/clustering leans on broad/common tags.
- **Specific tags = precision.** Stance/relevance pull leans on specific/rare tags.

**Mechanism — IDF, not LLM self-rating.** Derive specificity from observed tag frequency across the corpus.

### 6.3 Membership assignment & thresholds  **[DECIDED]**

Assignment for an incoming message:

1. **Reply reference → automatic membership.** Discord reply = member of X, bypass similarity math.
2. **Else embed topical tags, compare to each live conversation centroid:**
   - `sim ≥ τ_high` → member. Start ~`0.75` cosine.
   - `τ_low ≤ sim < τ_high` → ambiguous candidate. Start ~`0.45`.
   - `sim < τ_low` for every live conversation → **seed a new conversation**.
3. **Ambiguous band → ONE batched low-tier call.**

### 6.4 Decay & consolidation  **[DECIDED]**

- **Death trigger — recency TTL.** After a quiet window (~30 min) a conversation is dead.
- **Live-set bound — TTL + hard top-N cap per channel.**
- **Death ≠ memory write.** Junk conversations just evaporate. Only *salient* conversations get summarized.
- **Salience bar — both modes, toggle, default participated:**
  - *Default:* conversations Cova was in → med-tier summarizes into episodic memory + reconciles stances.
  - *Toggle:* also consolidate substantive conversations he witnessed.

---

## 7. Memory — Two Stores  **[DECIDED]**

- **Episodic recall** — pgvector over past exchanges. Retrieved by similarity.
- **Stances** — explicit, *mutable* rows keyed by subject (person/topic), with sentiment + note. Stored in PostgreSQL. Updated by the MED tier via sqlx.

---

## 8. Tone Problem  **[DECIDED — approach, not yet implemented]**

Default LLM pleasantness is RLHF-baked and sticky; adjectives won't beat it.

- Demonstrate voice in **example exchanges**, incl. Cova being neutral, dismissive, or bored.
- Explicitly grant permission to disagree, to not engage warmly, to give a flat reaction.
- Model choice matters — A/B local models for baseline sycophancy.

---

## 9. Prompt Layer  **[DECIDED]**

Named, single-purpose fragments composed at runtime by a Rust assembler from a `PromptSpec`. Use `include_str!` macros (`rust-embed` or `include_str!`) with optional disk hot-reload for dev.

### 9.1 Fragment taxonomy — four consumers

| Consumer | Tier | Needs |
|----------|------|-------|
| Tagger | low | tagging instructions + structural enum defs. **Persona-free.** |
| Gate | low | relevance-judgment instructions, `GateResult` contract. Persona-*light*. |
| Generation | high | full voice + register + injects. Where the personality lives. |
| Med jobs | med | task instructions (`stance_reconcile`, `summarize`) + enough identity. |

### 9.2 Voice — hybrid

`voice` = **static spine + dynamic retrieval.** Fixed hand-written core anchors consistency; pgvector retrieves Cova's *actual past messages* relevant to the current topic as few-shot examples.

### 9.3 Per-person — three layers, in precedence order

1. **Theatrical bit** — explicit, authored, user-ID-keyed performance overlay.
2. **Relational register** — automatic, emergent. Inject addressee's stance row + relevant episodic history.
3. **Core voice** — the constant default, always underneath.

**Guardrail:** the core voice is **never replaced**.

### 9.4 Theatrical bits — Cova putting on a voice

**Still Covabot — not a separate persona.** These are *bits*: Cova knowingly hamming it up.

```rust
struct Bit {
    trigger_user_id: UserId,
    trigger_condition: TriggerCondition,  // Present | AddressedOrAbout
    overlay_fragment: PathBuf,
    pull_modifier: Option<f32>,
    energy_default: Option<EnergyLevel>,
}
```

---

## 10. Context Budget  **[IDEA — sketched]**

Token budget filled by priority; trim from the bottom when tight.

1. Identity + voice (near-fixed, always)
2. Current trigger message + natural engagement framing
3. Active persona override / relational register injects
4. Rolling channel window (last N messages — **with usernames**)
5. pgvector-retrieved memories + relevant stances + dynamic voice examples

---

## 11. Build Sequence  **[DECIDED]**

1. Engagement state + pull/restraint model — fixes the daily responsiveness annoyance.
2. Conversation tagging layered on top to sharpen pull.
3. Memory (episodic + stance) — makes him feel like he knows people.

Don't make conversation tagging a prerequisite for the responsiveness fix.

---

## 12. Open Questions  **[OPEN]**

- **Context budget numbers:** actual per-section token allocations (§10). Calibrate during build against the chosen models' context windows.
