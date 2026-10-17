# Card compiler

This pipeline discovers every downloaded card definition and every pinned engine
card source. It converts a bounded subset of the TypeScript ability DSL into
`swu-card-ir/v1`, a typed data-only rules language consumed by the Rust simulator.
Its build-time backend emits executable Rust for supported programs; other
validated programs retain explicit interpreter or live-modifier fallback. It does
not execute TypeScript callbacks, embed JavaScript in Rust, or select behavior by
card ID. Adding a card that uses supported source patterns needs no compiler edit.

**Inventory, source conversion, and executable support are separate facts.**
Every discovered card appears in the catalog. A source-converted card can still
require an operation the Rust runtime has not implemented. The runtime must
validate all its abilities, keywords, types, targeting, and state before admitting
it. The current catalog is not an all-card implementation of SWU.

## Run locally

Use Node 22 and the normal repository setup. The parser uses the TypeScript
version already installed by the pinned `forceteki` dependency.

```sh
node compiler/build.js
node compiler/build.js --check
node --test compiler/test/*.test.js
```

`--check` recomputes all outputs and exits 1 if a generated file is missing or
stale, including when a new card or engine source appears. `--strict` exits 2 if
any known card remains unconverted; this deliberately does not pass yet. The
normal generation command succeeds with explicit unsupported entries so that a
new rule family cannot prevent already understood cards from being updated.

The optional `--source`, `--cards`, and `--out` arguments accept another engine
server directory, downloaded card-data directory, and output directory. For
example, a fixture source directory contains `game/cards/*.ts` and its data
directory contains `Card/*.json` plus `_mockCardNames.json`.

## Generated artifacts

- `generated/catalog.json` contains every nonmock card's full metadata, compiled
  abilities, keyword values, required capabilities, and source/data hashes.
- `generated/coverage.json` separates `sourceAstConverted` from `dataOnlyConverted`
  (plain bases and fully parsed keyword-only units with no source class, plus plain stat-only upgrades using the pinned generic `UpgradeCard` fallback), and lists
  all converted IDs, every blocked card, and blockers
  grouped by source pattern. `runtimeReady: null` means this report has not
  measured runtime acceptance.
- `generated/coverage.md` is a readable version of the source conversion report.

A rejected definition carries a pattern, file, line, and column. Partial
abilities may be retained for inspection, but `status: "unsupported"` rejects the
whole card. They must never be played as the subset of rules that happened to
compile. Unmatched sources and shared base files remain in the source inventory;
test mocks are counted separately. A compiler exception fails the build rather
than being relabeled as a normal unsupported pattern.

Provenance covers all engine server TypeScript and JavaScript, every card JSON file, the mock
list, compiler JavaScript, and its keyword-parser dependency. Outputs have no
timestamps or absolute machine paths, so identical inputs produce identical
bytes. The catalog also records the TypeScript parser version.

## Pending construction programs

Closed construction programs retain the order of local declarations, collection
iterations, constructor arguments, and property assignments. They distinguish
values and physical object references captured at construction from callbacks
invoked later. Conditional and nullish constructions evaluate only the selected
branch; native property factories retain each invocation rather than caching a
previous legality check. Private helper and callback programs permit checked
local mutation and assertions, with explicit argument and context bindings.
Unknown native helper names, properties, methods, and unmodeled writes still
reject the complete card.

These programs include pending selection, cost, ongoing-effect, lifecycle, and
class-definition contracts. They require a corresponding executor before runtime
admission. Complete token lifecycle programs may be staged under `sourceSupport`
while the existing native adapter card entries retain their reviewed boundaries;
staging does not count as a newly converted active card or executable support.

## Conversion boundary

The compiler lowers declarative action, event, when-played, on-attack,
when-defeated, and related registrations. It preserves costs, conditions,
targeting, continuations, dynamic amounts, and distinct sequential/simultaneous
effects. It understands bounded expression trees, enum values, readonly callback
locals, and an explicit query vocabulary. Supported trigger-menu `contextTitle`
expressions retain live scalar values in a bounded fragment list. Static text
helpers preserve engine formatting, including numeric keywords and multiword
traits; unknown helpers and unsupported dynamic expressions are rejected.

Phase stat modifiers capture bounded signed power/HP values when applied and
expire together at phase end. Constant stat modifiers support static self and
aura bonuses with bounded conditions and target filters. Arena count/existence
queries preserve their outer target separately from each queried card. General
live-stat calculations and stat-dependent constant conditions remain rejected;
the two pure stat-count callbacks described below have bounded native leaves.

Bounded arena lists, filtering, existence and length queries also support three
complete source shapes: conditional Ambush from another friendly nonunique unit,
optional exhaustion conditioned on another friendly space unit, and a leader
front action counting exhausted units across both players. Query candidates stay
separate from the chosen target and physical source. The front action retains its
resource and exhaustion costs, including the engine's no-effect confirmation,
and requires the exact native Spy definition in the selected pool. Whole-source
and printed-side guards reject other collection uses. Query programs use the
shared interpreter or live-modifier path; the leader's separate scalar attack
ability can use existing generated code.

Ongoing sources include actual bases and undeployed leaders for bounded complete
families: the 26-HP bases granting friendly leader units +0/+1 or +1/+0, and the
two-sided leader granting Raid 1 to friendly units costing at least three. A
leader uses its current side once; the deployed source retains its printed Raid
without granting itself another copy. Whole-source and printed side metadata
must match these supported shapes. Other base abilities and leader-front
constants remain rejected. These constant programs use the shared live-modifier
path in native and Wasm; runtime admission does not establish every interaction.

Constant and phase effects can grant or remove the supported core keywords.
Numeric Raid and Restore stack, while boolean keywords track presence. Live
numeric conditions are restricted to supported queries; keyword dependency
cycles and unsupported duration/value shapes remain rejected. Intrinsic
keywords require the engine's printed-text match: a keyword in the downloaded
metadata may only be mentioned or granted to another card.

Bounded native ongoing registrations support self `cannotAttack`,
`cannotAttackBase`, and all-zone `entersPlayReady` on complete ordinary units.
Ready conditions cover a literal round, a controlled in-play printed title, a
friendly nonunique unit, or an empty opposing arena. They evaluate immediately
before entry; importing or placing a unit preserves its supplied readiness.
Attack restrictions apply to action availability and target legality, including
Saboteur, while legal Overwhelm damage remains independent. A separate bounded
friendly-unit aura permits base targets during Ambush, preserving Sentinel and
the attacker's own restrictions.

Two pure live stat-count callbacks also lower to native leaves: other friendly
arena units with the source's printed title, and controller-owned discarded units
with a literal trait. Callback slots, local declarations, evaluation timing,
source defaults and the entire wrapper are checked before admission. Unknown
fields, registration-time captures, optional chains, stat-dependent predicates,
and unreviewed effects remain rejected. Auto and interpreter paths share these
native leaves; generated stat and attack code does not evaluate a second version
of their semantics.

Ordinary units and events can initiate one chosen-unit attack from a sole event,
When Played, or action registration. A typed, closed attack-properties contract
supports literal early power bonuses and Grit, Saboteur, Overwhelm, Raid, or
Restore grants. Candidate predicates may read bounded unit traits/aspects,
nonleader identity, and the initiating source's identity. Enemy, exhausted, and
attack-restricted units remain ineligible. Parent targeting precedes payment;
attack exhaustion follows defender selection. A parent exhaustion cost can make
its previously selected source ineligible before attack initiation resolves.

The nested attack retains the initiating ability's continuation and resumes it
only after declaration triggers, combat, attack-end triggers, and cleanup.
Unconditional modifiers apply before defender selection, including Saboteur
legality; their state-change window settles before the attack resolver rechecks
its attacker. Cancelling or passing that attack retains its early modifiers until
an actual attack completes or the recipient leaves play. These grants reuse the
versioned Support snapshot, with an explicit modified-attack kind and validated
physical donor. They use the interpreter fallback in Auto mode. General attack
factories, attack-end/on-attack initiation, exhausted-attacker permissions,
defender modifiers, altered damage, and unverified continuations remain rejected.

Two closed leader compositions reuse this machinery: a front-side native attack
action with physical resource/exhaustion costs, and a deployed other-friendly
Grit or Overwhelm aura. The Grit composition filters Heroism units. A bounded
ordinary event can also follow initiation with a fresh three-Advantage selection.
Its `ifYouDo` tests resolution of the initiating event: cancelling the later
defender picker still runs the continuation, while cancelling before initiation
or creating no initiation event skips it. Attack/death triggers finish before
the fresh recipient prompt. Other leader and continuation shapes stay rejected.

Closed arena-choice sources reuse the same menu interpreter for opponent arena
exhaustion on death and uniform three-damage arena batches on play or attack.
An explicitly legal Pass remains available without an effective event and
publishes a false result with no event payload. Global damage batches require
recorded global arena order at import and execution; existing registration
chronology guards still apply. Unrelated selected pools retain their legacy
snapshot compatibility without inferring missing order.


Bounded physical-card inventories support `some`, `every`, `filter`, `map`, and
`includes`, retaining order, duplicates, empty-array truthiness, and physical
identity. Each callback has its own card binding while the source and outer
target remain unchanged. Inventory equality and numeric coercion remain rejected.
The attachment-defeat families snapshot the full list, finish its primary
removals, and then perform one stat-defeat sweep. This does not implement arbitrary
list-target systems or grouped movement.

A shared live trait inventory combines printed traits and every active gain,
then removes every active loss. Interpreter and generated predicates, attached
trait checks, controlled-in-play trait queries, and leader-trait unions use that
inventory. Literal persistent self/parent gain/loss leaves require unconditional
constant registrations. Native ongoing forms cover a friendly Creature-like unit
aura across in-play controller and nonplay owner domains, and a live friendly
leader-trait union with a literal exclusion. All-zone sources retain their traits
in hand, deck, resource and discard zones. Trait producer cycles and unimplemented
source contexts reject before execution. Foreign-control imports remain rejected;
these contracts do not synthesize separate ownership history. Leader-trait union
programs cannot be Support donors or imported copied Support programs because a
leader recipient can create an unsupported feedback cycle.

Standard self-defeat registrations retain their source and controller through
zone movement and nested trigger windows. Standard multi-trigger aliases use
the same programs. Printed Coordinate enables supported constant and on-attack
abilities at three friendly arena units, including tokens; on-attack eligibility
is captured when the attack is declared.

Unit-token creation supports Battle Droid, Clone Trooper, Beast, Spy, TIE Fighter
and X-Wing tokens, one to five at a time, for either player, with explicit ready
entry. Token allocation precedes entry, and tokens leaving play stay out of
ordinary hands and discard piles. Scalar hand-discard effects let the affected
player choose their own cards. Their inherited default target is the opponent;
the helper's name does not imply a friendly target. Player selectors remain
rejected until their selected-player context can be preserved.

Force token state records each player's persistent token ID and whether it is
held. Gain, spend, and regain reuse that identity without allocating a card; the
two players remain independent across phase changes. Full-state export includes
`forceStateVersion: 1` and `players.<seat>.forceToken: { id, held }`, including
tokens outside play. Imports validate canonical, distinct IDs against the full
card inventory. Markerless legacy snapshots remain valid only for selected pools
that cannot reach Force abilities; held state and reachable Force abilities
require the reviewed native token in the selected pool.

Ordinary units support mandatory Force gain on play, attack and defeat, or
optional use on play and defeat. Multiple gain registrations retain separate
captured triggers and reuse the same token identity. Ordinary upgrades may use
their controller's Force on play before a bounded base-healing continuation;
attachment restrictions, printed stats and uniqueness still apply. Event use preserves prepayment
confirmation and cancellation when no Force is held. Supported continuations
include draw-two, self-ready, scalar unit damage, unit/base healing, unit phase
stats, and the opponent choosing one card to discard. Force is spent before the
continuation and remains spent even when it has no eligible or beneficial target.
Captured source identity survives departure and unique replacement. These
operations use the validated interpreter fallback in Auto mode.

Ordinary-unit constants may read their controller's current Force presence to
grant self Grit, Sentinel or Ambush, or a friendly power-only aura. Grit, Sentinel
and power update live when Force changes; Ambush registration is captured on
entry, so gaining Force later does not create an attack. Presence reads require
the same native-token and versioned-state evidence even when Force is absent.
Action costs may require the controller's Force, including alongside exhaustion
and bounded resource costs. Targeting and cancellation precede payment; a
confirmed no-effect action still pays. Payment checks the exact held token
identity before any cost changes and spends it once. Duplicate Force costs,
other payers, partial payment, and triggered additional costs remain rejected.
Both generated and interpreted programs use this shared cost boundary.

Force-dependent HP changes, other query receivers, groups, hooks and
continuations remain explicit rejections. Source conversion and runtime admission
remain separate.

An ordinary upgrade may optionally give one Weakness token to any unit on play.
The upgrade attaches and its printed stats apply before that choice. Its
controller chooses even on an enemy host; the new token belongs to the chosen
recipient's controller. Declining creates no token. The selected pool must include
the reviewed native Weakness definition. Other upgrade Weakness registrations
remain rejected.

Ordinary events may give one or two Weakness tokens according to their controller's
base trait. A bounded optional play/death pair combines one damage with one
Weakness on the same chosen unit: Shield prevention affects the damage only, and
the token attaches before the combined lethal check. Both token identities in a
two-token window are allocated before attachment. Ordinary play may also create
an exhausted Beast before a mandatory friendly Weakness choice, or give Weakness
before exhausting the surviving captured recipient if it is unique. Physical
death can precede a later step while death triggers wait for the parent ability.
Beast creation requires the reviewed Beast definition in the selected pool.
These bounded forms use the shared interpreter in native and Wasm Auto mode.

Physical `hasExperience`, `hasShield`, and `hasWeakness` predicates require their
reviewed native definitions in the selected pool, even when no token is created.
Bounded scalar leader actions and optional unit-side attacks can give Weakness;
the complete two-sided source, costs, target filters, and continuations remain
admission boundaries. The bounded Credit lifecycle and payments described below
execute through the shared runtime. Advantage remains compiler-unsupported
because its pinned source contains two classes, while an exact native adapter
executes its complete reviewed lifecycle; native admission is counted separately
from source conversion.

Advantage is a physical +1/+0 token upgrade owned by the recipient's controller.
Bounded scalar creation supports literal amounts up to five and the live opposing
unit count; bounded multi-selections retain their creation batch and click order.
All token identities are allocated before attachment. Unknown creator, owner,
batching and amount contracts remain rejected. The selected pool must contain the
exact reviewed native definition, even when a particular branch creates no token.

At attack end, Advantage registers on upgrades of both involved units after
combat damage handlers and before pending defeat subwindows physically move the
units. Tokens on lethal hosts can therefore leave an ineffective captured owner
bucket when another trigger can resolve. Mandatory token self-defeat shares the
ordinary attack-end/death ordering window and runs once; Advantage granted while
that window resolves remains for the next attack. Support may copy the bounded
ordinary attack-end registrations, including their watcher queries and source
identity rebound to the supported attacker. Copied registrations survive donor
departure and use the existing versioned Support snapshot/import contract.

A bounded self-resource operation moves an existing event or defeated ordinary
unit into its owner's resources, exhausted, without allocating a new card.
Supported sources are standalone events, optional death registrations with a
live discard-zone guard, and optional death-time Force use followed by self
movement. The Force continuation preserves the source's current legal nonarena
zone; it does not acquire an implicit discard condition. Already-owned resources
keep their readiness, and destination capacity is checked before removal.
Other targets, ready entry, resource defeat and controller changes remain rejected.
This operation uses the validated interpreter fallback in Auto mode.

Bounded simultaneous groups preserve separate event preparation and resolution.
The runtime supports selected combinations of damage, healing, tokens, readiness,
and phase modifiers, with independent nested selections. Up-to selections retain
click order, toggles, and zero selections; unlimited selection is currently
restricted to the two bases. A single phase-effect call with several modifiers
remains one lasting effect. Group admission rejects unsupported continuations,
ambiguous repeated damage, and reference scopes the runtime cannot preserve.
Supported draw/discard groups prepare choices before resolving their effects.
Mixed damage with hand effects or newly created unit tokens remains rejected.
The runtime checks its bounded storage and fails explicitly on overflow.

Two independent named phase-stat selectors also support ordinary-unit On Attack
registrations. The first selector may be mandatory; the second remains optional.
Each uses a static unit domain in either arena or one named arena, with either
controller eligible. Both choices are captured before applying the modifiers,
and the pending attack resumes after resulting defeat triggers. Bounded literal
negative stats, including negative zero, retain their source meaning. Dependent
names and additional attack predicates remain rejected. Named groups use the validated interpreter fallback in Auto mode.

Two independent named damage selectors support ordinary-unit When Played and
On Attack registrations. The source controller chooses a mandatory friendly
unit first and an enemy unit second, with static arena domains and one or two
points of preventable ability damage. Both choices finish before either damage
event resolves. Shared Shield replacement and lethal-trigger handling preserve
recipient identity and the pending attack; a missing enemy does not suppress
friendly damage. Dependent names remain rejected.

A separate On Attack pair selects an enemy ground unit for two damage, then either
base for two damage. Both choices belong to the source controller and finish
before damage resolves. A missing ground recipient skips only that effect. The
pending attack continues through captured mandatory death effects, including
automatic effects after a base defeat has latched the winner. Other unit/base
combinations remain rejected.

Arena keyword events preserve one lasting-effect helper with two factories as
a different operation from two simultaneous helpers with independent recipient
lists. Both capture global arena order before applying phase grants or losses;
combined keyword and stat capacity is checked before publishing the result.
Ordinary On Attack may grant Restore to other friendly units of a fixed trait.
The source retains its separately captured intrinsic Restore trigger. These
operations use the shared interpreter in native and Wasm Auto mode.

A bounded opposite-chooser pair lets the opponent choose their unit, then the
source controller choose a friendly unit. Ordinary play registrations deal three
damage to each or defeat each; the event variant defeats nonleader units. The
opponent selector delays targeting until the original actor has paid, even when
that domain is empty. Effects wait for both choices, then replacement and death
priority return to the original actor. Other chooser layouts remain rejected.

Bounded mixed named events combine ready with three damage, exhaust with one
Shield on a friendly unit of printed cost three or less, or one Shield with a
-3/-0 phase modifier. Both recipients are selected before effects and token
allocation. A missing first target delays later selection until after payment;
a legal but already ready/exhausted first domain keeps later selection before
payment. Cancellation preserves card identities and resource state. Unsupported
mixed pairs remain rejected rather than running a partial effect.

Bounded ordinary-unit play and defeat registrations can give one Shield or
Experience to each matching friendly arena unit. Supported live filters cover
Shielded keyword presence, physical source exclusion, the Rebel trait, and
already-damaged units. Each registration allocates its entire token batch before
attachment, preserving ground-then-space recipient order and recipient controller
ownership. Captured play and defeat registrations remain distinct through unique
replacement. Intrinsic Shielded appends to the existing token chain, preserving
older Shields for damage replacement. These forms use the shared interpreter in
native and Wasm Auto mode; other token-list shapes remain rejected.

It rejects unknown fields, custom class state/methods, constructors, accessors,
module execution, activation limits, unsupported effect helpers, and mutable or
side-effecting callbacks. Other constant abilities, arbitrary event watchers,
other lasting effects, custom inheritance, many zones, and many other rule families remain
work items in the coverage report. The Rust operation set is narrower still.
Source conversion of a simultaneous effect does not imply runtime support for
every operation or interaction in that group.

Continuation factories that capture a target across new targeting, or dynamic
values across multiple effects, are rejected until the IR has explicit capture
slots. Unused local initializers are rejected so their possible side effects
cannot disappear. General JavaScript closure semantics are not implemented.

The three original leader definitions are compiled from their actual source,
using exactly the same path as new cards. Native engine tokens, such as Shield,
may have an independently verified runtime implementation while their source
class still appears as unconverted; the source report does not count those as
AST-converted.

## Extending support

1. Choose a blocker family in `generated/coverage.json` and inspect the pinned
   engine helper semantics, including defaults and legal-target behavior.
2. Add a typed lowering rule. Preserve distinctions such as basic units versus
   token units, optional targeting, event timing, and conditional continuations.
3. Add runtime support or leave an explicit runtime rejection. Never weaken the
   importer merely to increase an accepted-card count.
4. Add a small synthetic-source compiler test and a real engine differential
   fixture for the newly executable behavior. Run the relevant Rust tests and
   trace comparison before claiming gameplay support.
5. Regenerate all artifacts and run `--check`. The resulting coverage changes
   are reviewable alongside the implementation.

## Compiler-only event context representation

`eventResult` preserves reads from a lexical `AbilityContext` object's `events`
or replacement-resolved `resolvedEvents`. Its `contextBinding` identifies the
callback argument by source position and parameter index, records its role, and
keeps binding time (`callbackInvocation`) separate from read time
(`expressionEvaluation`). Captured outer callback arguments retain their own
binding when a child property factory or selector runs. These are context-object
references, not substitutions of the current target or numerical result register.

Indexed reads retain the complete field path and every optional-access segment;
optional missing values must propagate before surrounding operators are applied.
Parenthesized optional event-chain boundaries remain explicit compiler rejections
until their representation is verified.
The pure damage reducer retains the resolved-event filter, missing-value zero,
and sum seed. `resultIncludes`, `resultCardType`, `resultSharesBaseAspect`, and
`contextRef` retain membership, enum-type tests, and the discarded-card/base
comparison without losing the captured physical card. Generated-token lists
refer to IDs from the originating creation event, including first-element reads,
not a live token inventory. Dynamic resource-pair token counts retain their
floor/division expression. Existing admitted scalar `totalDistributed` programs
keep their current representation.

These new nodes are compiler output only. Runtime callback bindings, replacement
results, created/discarded identity storage, optional continuations, dynamic
creation capacity, and Plot remain explicit engine backlog. No gameplay support
is implied by the compiler's supported status; the current Rust loader rejects
these new nodes and continuation fields.

## Bounded native selectors

Rust retains typed `cardSelection` descriptors for single-card selection with
literal card-type unions, zones and controller filters. Candidate predicates
have their own binding; they do not replace the initiating source or player.
The admitted ordinary unit/event subset includes a bounded opponent chooser for
damage, defeat or one Weakness, using the existing suspended-group protocol to
pay event costs before the opponent chooses and restore the initiator afterward.
Front leaders can be readied through the ordinary exhaustion mutation helper.

Unknown fields, dynamic selection counts, explicit must-change-state overrides,
resolution-time selector conditions, captured candidate scopes, arena upgrade
inventories and physical base-token inventories remain rejected. Complete source
metadata is required; expanding a type filter does not admit unrelated leader,
attachment or history registrations. Existing action-window snapshot support is
unchanged; the new pending selector prompts have copy/action-set assertions.

## Event hooks and phase history

`eventHooks` retains a single registration with alternative named event predicates
or standard aliases. Optionality, collective triggering, source-zone filters and
limits belong to that registration; alternative hooks share its limit. Event
paths retain callback argument identity and optional-access segments, including
outer contexts captured by nested continuations. Damage/defeat source enums use
the pinned `IDamageOrDefeatSource` definitions.

`stateWatchers` declares named `StateWatcherLibrary` factories, shared at game
scope and reset on `onPhaseEnded`. `historyQuery` retains the exact pinned
watcher API operation and arguments. Its meaning includes the implementation's
saved physical IDs, in-play IDs, controllers, LKI and default filters; it is never
a live arena query. `historyAttackCheck` likewise retains the pinned
`AttackHelpers` survival or defeat test. `capturedHistory` binds a readonly
value at its declaration in the identified setup/callback invocation.

Typed `historyHelper`, `dataLambda` and `callbackProgram` nodes retain pure
helper/predicate arguments, block scopes, readonly locals, branches, switch
cases, templates, optional member accesses and finite named query calls.
They contain data operations, with no executable source strings. Unsupported
statements and unknown query operations still reject the complete card.
Rust executes a bounded subset of these contracts: complete ordinary-unit
registrations using the reviewed entered/played watcher predicates and play
counts, plus unconditional attacker-owned `whenAttackEnds` registrations and
the exact defender-defeated watcher check. Phase history records physical
identities and controller-at-event, persists after departure, and resets at phase
end. Import requires versioned exact history when a selected card can read it;
arena contents cannot reconstruct missing history. Attack history follows the
original attacker and defender through suspended trigger windows and departure
or re-entry, with attack cleanup after the shared combat event window.

Other event hooks, saved callback values, watcher operations, registration limits
and conditional attack-end captures remain rejected. Conversion is not a claim
of runtime admission or gameplay verification. Focused Rust assertions exercise
the admitted subset; differential parity for these additions remains unverified.

Damage-modification grants use the same pinned registration contract with the
recipient as ability source and the grantor retained separately. Explicit ordinary
activation limits use the shared strict limit parser. `unresolvedContinuation`
retains `ifYouDoNot` and its own `resolvesTriggers` window; the pinned condition
checks event resolution/replacement status, rather than a numerical result.
`unlessSelection` retains both systems, button labels, chooser, highlights and
unresolvable display flag. Its default branch is automatic when the unless system
cannot fully or partially resolve. These remain pending executor operations.

The complete ready/conditional-Spy event registration can project current unit
play from existing exact phase-played IDs and current-incarnation qualification.
Both histories are required: an old played bit after departure/placement is
insufficient, as is qualification from deployment or token creation. The target
may belong to either player; Spy creation belongs to the event player and is
independent of whether readying changed the target. General captured event lists
and other history callbacks remain rejected.


## Runtime admission and conformance

```sh
target/debug/kessel catalogue compiler/generated/catalog.json --out-pool compiler/generated/runtime-pool.json
target/debug/kessel check traces.jsonl.gz --catalogue compiler/generated/catalog.json --pool compiler/generated/runtime-pool.json
# in Holocron, next to this checkout (records forceteki games):
node bots/omni/rs/make-compiled-fixtures.js --pool ../kessel/compiler/generated/runtime-pool.json --out replays/compiled-card-fixtures
```

`catalogue` emits a deterministic admission report, including named rejection
reasons. `check`, `prove`, and `bench` accept `--catalogue` and an optional matching
pool. Omitting the catalog retains the original strict pool behavior.

Closed whole-source contracts also admit bounded card restrictions, adjusted
resource payments, Disclose continuations, self-play cost reductions, modal
choices, first-strike combat and selected hand/discard resource or bottom-deck
moves. These validators check the complete source registration and metadata;
the presence of an IR node alone does not admit its containing card. Unsupported
companions and unreviewed source contexts remain rejected.
Self-play reductions read their validated live unit/leader counts when calculating the total
play cost, including aspect penalties, and floor the result at zero. First-strike
combat settles the attacker's damage and death triggers before calculating a
surviving defender's return power. Modal choices preserve the chooser and resume
the selected branch through the ordinary program interpreter.

Two closed ordinary-event cost contracts retain their executable event after
extracting the cost registration: friendly arena leader units reduce an up-to-three
Experience event, and friendly Rebel units/leaders reduce a Draw 2 event. The
Rebel inventory uses live traits and counts the undeployed leader once; attached
ordinary Pilots retain upgrade mode and do not count as units. Leader-upgrade
modes remain outside admission. These reductions apply to the source's play cost,
not ability costs or card-effect resource payments.

The fixture generator records real engine games across four bot pairings plus
explicit effect/continuation decks, and preserves the exact pool beside them.
Its observed-card/effect report describes what ran in JavaScript; only a passing
Rust `check` establishes differential agreement for those traces. Admission of a
card alone does not establish conformance of all that card's interactions.

Repository CI regenerates conversion/admission artifacts and runs the existing
checks; it does not use model calls or alter engine source. Use `npm run
cards:compile`, `npm run cards:check`, and `npm run cards:test` locally. New syntax
within supported rules converts automatically, while unsupported families remain
named work items.

The separately configured daily local Codex job is authorized to implement and
push tested conversion/runtime increments. That local schedule is not repository
CI configuration. Each new executable family still requires real-engine
comparison before its coverage is claimed.

See the [card catalog and reference-test guide](../cards/README.md) for card,
behavior, and test queries; reusable Karabast generation; and native/Wasm checking.
The compiler remains the authoritative source of metadata and ability IR, while
`../cards/` joins it with runtime admission and named semantic cases. Accepted
recordings live in `../cards/reference/fixtures/` and are checked by both runtimes
in CI. Work chronology and review journals do not define card coverage.

The original strict-pool and compiled-runtime integration corpora remain useful
smoke tests for their loading modes. Current counts come from `npm run cards --
summary` and `generated/runtime-admission.json`, rather than historical batch totals.

Follow the [conversion skill](../.agents/skills/swu-card-conversion/SKILL.md)
for implementation, source-derived witnesses, and independent review requirements.

## Bounded Credit and attachment payments

The exact native Credit definition supports four complete sole registrations:
ordinary unit entry creating one Credit, and ordinary events creating one or
three. Creation allocates fresh physical identities as one batch before ordered
BaseZone entry. The ledger retains zone order, controller/owner changes and
retired physical metadata; defeating a Credit never recycles its identity.
Other printed Credit registrations and modified or blanked Credit states remain
rejected. These contracts do not imply support for every Credit card.

The shared resource-cost boundary applies Credits after ordinary reductions and
aspect penalties for normal play, Smuggle, action costs and adjusted card-effect
payments. It preserves mandatory minimums, amount selection, skip and inherited
cancellation rules. Earlier costs settle before later payment stages. A prompt
retains its evaluated cost, physical source and chosen attachment host. Paying
entirely with Credits still resolves a zero-amount resource event, so the
payment's success continuation can run. Unadjusted resource effects retain their
separate semantics. A cost that becomes unavailable during a child window fails
explicitly when recovery is unsupported.

Stable selected-pool snapshots require `creditStateVersion: 1` and explicit live
`credits` and canonical `retiredCredits` references for both players, including
empty lists. Legacy snapshots cannot establish unknown Credit evidence. Native
state copies retain pending payment and child-window continuations; JSON export
marks those frames and action-window import rejects them.

Three complete upgrade contracts reduce the price by one when the physical
attachment target is Imperial, Mandalorian or unique. Target availability and
payment use that particular host, including a zero floor. Target-free printed
cost queries remain undiscounted, and Smuggle retains its printed alternative
cost. Broader player cost modifiers, captured attachment lists and unsupported
companion registrations remain rejected.

## Bounded controlled-title and physical-attachment queries

`hasControlledInPlayTitle(player, literalTitle)` tests exact printed title equality
across controlled arena cards (including upgrades on opposing hosts), the player's
base and undeployed leader, and held Force. Force-title programs require the
reviewed native definition and explicit Force state. Live Credits participate in
controlled in-play title and trait queries through their BaseZone ledger; retired
Credit identities do not. Fortify and printed attribute overrides remain outside
runtime admission. Title-dependent source
registrations require complete ordinary-unit metadata. Reviewed consumers are
persistent self constants with the query condition, mandatory entry self-ready,
optional On Attack one-damage unit selection, and optional death Experience
selection. Conditional event consumers use an exact title query (or two joined by
OR); query placement, targeting and continuation topology remain validated.
Other source conversions stay runtime-rejected.

`hasAttachedTrait(card, literalTrait)` and
`hasAttachedNativeToken(card, "shield")` inspect the unit's unified physical
attachment chain, regardless of attachment controller. Native token identity is
separate from the Shielded keyword or an ordinary upgrade's title. Interpreter
and compiled Rust expressions call the same typed helpers. Constant registrations
retain the live modifier scheduler and reject whole-program forced AOT. The compiler lowers
only synchronous, literal, canonical card predicates and the reviewed complete
Resistance helper topology.

Friendly-ground unshielded Shield creation has a separate complete-registration
and metadata gate. It reuses the ordered token-batch executor; the source's own
entry is eligible. Its program retains the existing interpreter fallback. Focused
Rust tests cover these contracts; engine parity remains unverified under the
Rust-only validation constraint, so these are locally reviewed conversions rather
than newly established reference coverage.

Bounded `floor(baseDamage / literal)` expressions accept only the current player's
or opponent's nonnegative base-damage field and a positive integer divisor up to
65,535. Both evaluators use integer division on that proven nonnegative domain.
Naked division, other numerators, malformed argument counts and zero, negative or
fractional divisors remain rejected.

An ordinary unit On Attack selector may give one Experience token to another
nonleader unit sharing a trait with any friendly in-play leader card. The live
leader trait union includes undeployed base-zone leaders and controlled deployed
leaders, uses nonempty deployed-side traits when present, and deduplicates traits.
The selector remains optional and can select an opposing unit. Other uses of the
leader collection remain rejected.

The bounded live keyword inventory copy aura grants each other friendly Spectre
unit its source's current keywords. Its explicit copy domain is Sentinel,
Overwhelm, Saboteur, Ambush, Grit, Shielded, Raid, Restore and Hidden. Boolean
presence deduplicates and numeric values stack, including Raid presence at zero.
Source and recipient keyword losses remain independent; source departure,
attachment removal and phase expiry remove the corresponding live grants.
Unknown donor inventory keywords and copy dependency cycles fail explicitly.
A recipient may independently possess a keyword outside this copy domain.
This source conversion has focused Rust evidence; no new engine recordings were
made for this increment.


Fixed arena reductions preserve a live own/opponent arena query and zero seed.
The admitted shapes sum physical attached upgrades on enemy hosts for optional
readiness, or sum friendly unit damage for conditional event damage. Attachment
controller does not change the host-based count; source and target candidates
remain distinct. A pure private event helper can provide the live damage sum,
including the preserved dynamic selector prompt in IR metadata. Raw IR must retain the complete
reviewed source metadata and registration. Sums exceeding compact effect storage
fail explicitly. These forms use the shared interpreter fallback.

Maximum-power reduction also converts the two-sided Overwhelm leader source,
retaining ties, the zero seed, and its deployed side. Runtime admission currently
rejects that leader in the complete catalog because an existing keyword-inventory
stat aura can feed back into the power-dependent grant. Reduced-catalog Rust
diagnostics cover the acyclic primitive; they are not admission or engine
conformance evidence for the complete catalog.

The bounded Support family uses the pinned engine's modified-attack lifecycle.
An intrinsic Support keyword offers another ready friendly unit an ordinary
attack. At initiation, the attacker gains a snapshot of the donor's keywords
excluding Support, and copies its admitted constant and On Attack registrations.
Copied registrations use the attacker as their source. Donor departure and later
keyword changes do not alter the snapshot. Source-bearing donors currently require
at most one ordinary constant or On Attack registration; other donor topologies
and effects granting Support remain explicit runtime rejections.

The pinned engine applies these grants before defender selection. Passing or
canceling at that prompt retains them until an actual attack completes or their
target leaves play. The simulator preserves this behavior, including across
phase boundaries. Storage is bounded to 32 lasting grants with at most 80 copied
registrations each, checked before publishing an effect. Support state adds 5,992
bytes of payload to the fixed Copy state; the combined runtime is 28,032 bytes
including resolved-event and maximum captures. Card storage remains 36 bytes.
Imports from a selected Support pool require `supportStateVersion: 1` and a
`supportGrants` list containing the actual keyword and admitted-program snapshots.
Pending Support attacks cannot be imported as action-window positions. Legacy
imports remain available to selected pools that cannot reach Support. This is
source-derived Rust coverage; no new engine differential recordings are claimed.

Bounded resolved-event captures distinguish actual damage from the legacy
numerical effect counter. Actual damage is HP-capped and zero when a Shield
prevents the event; a step retains its source/program/instruction identity through
nested death continuations. A pure event-window reducer may count successfully
defeated enemy units after a complete arena defeat, excluding attachments and
separate framework deaths. Defeat registrations may read pre-resolution
last-known power, captured for the entire batch before attachments or sibling aura
sources leave play. Controlled trait conditions share the complete controlled
in-play domain with title queries, including the base zone, while excluding the
specified physical source and respecting a deployed leader's printed traits.
These bounded forms retain interpreter fallback where scheduling requires it.

A bounded leader action/attack pair supports selecting the friendly unit with
maximum power among units of one fixed aspect for one Experience token. A
complete conditional factory is lowered into an explicit capture after cost
payment, with maximum value, tie count and sole physical recipient retained in
fixed state. A unique maximum resolves automatically; ties select live eligible
units at the captured maximum. Power is floored at zero by the pinned unit
helper, so negative raw power remains a zero-power candidate. Captures survive
state copies and suspended resolution contexts and are included in hashes.
Other source callback capture shapes remain rejected. This operation uses the
interpreter. Its capture adds 72 bytes in isolation and 112 bytes in the combined
layout, including padding; the combined State is 28,032 bytes and Card is 36 bytes.

An unseeded live friendly-power maximum in a scalar target comparison lowers to
an existence query that keeps the outer candidate separate from queried units.
The empty list produces false. Selector-header IR metadata retains the exact
maximum as decimal text, or `-Infinity` for an empty list; that text cannot be an
effect amount. The complete two-sided leader includes intrinsic Support execution;
Rust integration tests exercise deployment and a copied On Attack registration
using the recipient's power. Isolated action diagnostics remain separate from
those complete-card witnesses.

Pure quantity factories also have compiler-only IR forms. `createTokenUnitsDynamic`
retains a resource-count floor quotient at token-system property evaluation;
`discardOwnHandPerPlayer` retains the ordered `game.getPlayers()` collection and
an explicitly bound `perPlayerAmount` evaluated for each affected player. Its
hand-size expression is `max(0, affectedPlayer.hand.length - N)`, and the affected
player chooses their own cards. These are separate operations from fixed token
creation and fixed hand discard, and are not runtime-admitted.

Collection projections retain distinct-title and distinct-aspect counts over
friendly arena units, plus aspect-icon multiplicity over arena units or the
source's exact controlled in-play unit/upgrade domain. Multiplicity includes
repeated aspect icons; a distinct count uses value equality, including titles
without subtitles. Executing-callback stage checks reject earlier captured arena
snapshots. Phase modifier values remain captured at application, while constant
conditions and selector predicates query at evaluation. These new query forms are
compiler conversions pending runtime implementation and engine verification.

Compiler conversion can precede runtime implementation. Bounty registrations and
`gainBounty` modifiers retain the keyword's opponent collection context, default
optionality, defeat/capture event hooks, collection event, and no-legal-effect
resolution rule. An attached grant explicitly rebinds the ability source to its
recipient. `capture` retains its captor expression, non-leader target type and
out-of-play flag. These representations do not implement bounty collection,
capture zones, rescue, or leaves-play timing in Rust. Unsupported runtime schemas
and capabilities remain rejection boundaries until the separate engine phase.

Cost, resource and token systems also have compiler-only representations. Self
play-cost registrations retain all-zone activation, physical-source and printed
card-type matching, and cost-adjustment stage. Player cost modifiers retain their
filter, match/attachment predicates, optional cloned limit, resource-cost category
flags and callable values. `boundCallback` distinguishes the adjusted card,
adjuster's source player, source card, attachment target, remaining cost and actual
payment context; these roles do not share the ordinary selector candidate slot.
Cost selection records a separate `costSelection`, helper prompt, full-payment
requirement and ability-cost timing. Earlier captured arena and upgrade lists
remain rejected across callback stages.

Adjusted card-effect resource payments retain their payer, Credit/cost-adjuster
support and noncancelable payment stage. Unadjusted payments retain resource
exhaustion as a cost. Credit creation defaults to the acting player and enters the
base zone; a direct setup-time controller reference retains a registration
snapshot. Selected-player resolver bindings override helper target defaults only
when no explicit target was supplied. Advantage grants preserve creator identity,
recipient-controller ownership and one batched creation event. Advantage
distribution instead retains a separate creation event per recipient and its
aggregate result. Phase player-cost effects preserve one lasting-effect
application with expiry at phase end. The bounded Advantage grants and Credit contracts described above execute in Rust;
other resource/token forms still need runtime implementation and native token
contracts. Catalog conversion alone does not admit them for play, and executed
source-engine verification remains pending.

Extended resource/token systems retain explicit resource-controller and readiness
properties, top-deck card/list queries, ordered token-controller arrays, and token
entry systems. Entry effects are forcibly retargeted to each entering token after
entry and before the state check. Resource readiness remains a contingent event
and uses the original complete system target list. Broader complete top-deck
programs use `resourceCardSystem`, keeping existing runtime-facing programs
unchanged. Self aspect-penalty registrations reuse the typed cost-adjuster schema.
Callable ability costs remain `costFactory` programs evaluated on each engine
`getCosts` call. These lowerings do not by themselves establish runtime admission;
closed ordinary resource-entry forms have the bounded runtime support described
above.


## Further compiler-only contracts

Deck search retains search scope/count, selected and remaining card bindings,
selection predicates, ordering, bottom-deck randomization and whole-deck shuffle.
Rust admission diagnoses a closed literal-count/filter/selected-card subset using
strict typed contracts, but still rejects the complete containing card. The native
default bottoms unchosen cards **in random order**, even when the player takes
nothing; a whole-deck search always shuffles regardless of `shuffleWhenDone`.
Neither stable deck order nor a human-selected permutation implements that rule.
Execution needs an explicit randomness contract (including imported RNG evidence
or chance outcomes and search treatment), plus search/display continuations.
Empty or one-card fixture decks do not justify whole-card admission. Dynamic
counts, captured private callbacks, custom handlers and changed defaults remain
separate explicit rejections. This diagnostic foundation adds no State fields,
playable programs, card admissions or behavioral parity claims.
Play systems retain play type, type restrictions, ignored requirements, cost
adjusters and entry effects. Repeated hand/discard plays reselect after each full
play and its triggers; deck multi-play uses the selected order and free nested
plays. Object-shaped simultaneous/sequential systems retain their resolution mode.
View/selection, reveal and deck movement remain distinct pending operations.

Disclose preserves aspect multiplicity, target-player selection context, disclosed
card event data and the triggered no-match masking protocol. Rust admits closed
ordinary-unit/event Disclose forms with supported `ifYouDo` continuations.
Delayed systems
retain application context, named event predicates, matched targets, duration and
application limits. Optional and random aggregate systems retain child target
binding and native-default fallback; random selection keeps its cached result and
rollback-confirmation requirement. Delayed and random aggregate nodes do not
supply a Rust executor.

The two supported shared BaseCard definitions are compiled from their pinned
source, with inherited registration and source provenance retained. Subclass
setup overrides and unsupported abstract definitions remain rejection boundaries.
The stopped cohort is excluded before AST parsing by source-exclusions.js; its
historical definitions are preserved in Git but are not active conversions.

General system-array/property factories remain a representation decision. A
static child-system property may capture a value before preceding effects
resolve, while a child property factory evaluates later. A future construction
contract must distinguish these stages and preserve saved physical identities.
Dynamic system dispatch and native token constructor/lifecycle behavior also stay
unconverted. Runtime admission, Support hash coverage, State/search-copy cost and
engine verification remain separate follow-up work.


Pure array factories use the existing explicit callback/capture convention via
`systemArrayFactory`. Each native property-generation invocation gets its own
frame: readonly declarations and computed static child properties are captured
in source order before child resolution. Physical cards/players retain identity;
scalar values remain saved values. Deferred predicate callbacks retain lexical
bindings. Ordinary child-system lowering still validates every represented field.
Existing admitted factory forms keep their old lowering. Nested system property
factories, dynamic system dispatch and returned system objects stay rejected by
this bounded form; they need a richer construction graph rather than a live-read
substitution. This compiler-only node adds no runtime execution.

### Ordinary upgrade-granted trigger aliases

The Rust loader executes bounded `attachedAbilityRegistration` wrappers for
`onAttack`, `onDefense`, and `whenDefeated`. Closed wrapper fields must agree on
ordinary upgrade mode, physical parent binding, host controller, grantor presence,
and one enabled standard trigger alias. The complete upgrade metadata and printed
stats must agree. Supported bodies use the existing typed executor; unsupported
children reject the whole card. Keyword mentions in upgrade metadata do not
become intrinsic keywords.

Each physical attachment contributes its own host registration in attachment
order. Declaration and defeat windows capture the host source/controller before
handlers remove attachments. Captured triggers survive grantor departure;
departure before capture removes the grant, and host reentry does not restore a
previous attachment. Attack and defense grants share the existing owner-order
prompt. Arbitrary event hooks, action grants, gain conditions, activation limits,
pilot modes, parent/grantor reads inside the host body, and unimplemented leaves
remain rejected. No persistent State storage is added.

Attached constant contracts additionally support unconditional literal parent
trait gains/losses through the shared live trait inventory. A bounded host-granted
constant can give other friendly units of one literal trait constant stats; its
source/controller come from the host and its lifetime comes from the physical
upgrade. The complete wrapper is required, so this does not widen older bare
`attachedContext` forms. Optional parent-title checks have a bounded play-time
conditional Shield selector; a departed parent makes the comparison false.
Leader conversion, cost-aspect grants, blanking, dynamic modifiers, and unknown
wrapper fields remain rejected as complete-card blockers.

Bounded defeat-event hooks preserve one typed `onCardDefeated` predicate and an
optional standard When Played alias. Their closed predicates read pre-handler
LKI unit/non-token type, LKI controller relative to the source controller, and
whether the defeated physical card differs from the source. These are capture
predicates, not a general event-expression executor. Registrations support scalar
Experience grants, self Sentinel for the phase, own-base healing, or Battle Droid
creation. Every source registration and sibling ability must validate; only ordinary
nonleader unit metadata is accepted. Limits, collective triggers, event-derived
effect targets, dynamic continuations, nested grants, and all other hooks reject.
Capture reuses the simultaneous defeat window before any sibling leaves play, so
captured effects survive source defeat and retain their controller. A compact inline physical registration sequence preserves cross-arena unit and
ordinary-upgrade listener chronology, including leave/reentry and reattachment.
Versioned `registrationOrder` snapshots must supply exact membership, arena
projections and host/attachment order when the selected pool can reach defeat
hooks or granted death registrations; legacy imports never infer that evidence.
The inline State grows by 256 bytes (the 260-byte sequence absorbs existing
padding). Rust source-derived fixtures cover these boundaries; parity
for these additions has not been checked by executing the source engine.
