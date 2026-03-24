# Annex F — Black Swan Resilience

## Mesh-Failsafe

Global internet loss triggers automatic fragmentation: central ledger splits into 64 autonomous satrapy nodes. Local HF/mesh radio maintains humanitarian (Tier 1) distributions using a Degradation Coefficient (Cd) to estimate activity until resync.

## Ghost Nodes

Ledger shards hosted in undersea and orbital locations. The system survives as long as three fragments exist anywhere on Earth. Makes the protocol physically undeletable.

## Quantum Ratchet

Automatic migration to post-quantum cryptography (PQC) upon detection of a hash collision. 72-hour vetting window for new algorithm deployment.

## Orphan Node & Shared Burden

When a node loses all connectivity:

1. Neighboring Node-Beta projects "thin client" into orphan sector via mesh
2. Shared Burden Coefficient applied: V_final = V_base x (1 - P_orphan/P_total x sigma) where sigma is mesh overhead penalty
3. Results in a small tax on the helping node, but audit continues unbroken

## Cynical Reconciliation

On resync after blackout:
- Oil: lowest verifiable volume accepted (TEE + SAR + physical)
- COLP: highest verified biometric count accepted (no one denied life-support)
- Gaps: covered by Collective Defense Fund