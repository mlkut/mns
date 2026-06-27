// SPDX-License-Identifier: MIT
pragma solidity ^0.8.33;

/// @title MNSRegistry
/// @notice Decentralized name registry on Rootstock.
///
/// Architecture
/// ────────────
/// Ordinal space is divided into fixed-size Batches of BATCH_SIZE (256) ordinals each.
/// Anyone may register a new Batch (consuming a rate-limit token).
/// Each Batch has an owner and a default NS (authoritative name server RFC 1035) that resolves
/// for every ordinal in [ordinal, ordinal+255] except ones overridden with an Entry.
///
/// Individual ordinals can be overridden via Entries. An Entry takes precedence over its
/// containing Batch for both owner and NS resolution. Entries are permanent once
/// created; the entry owner may update but not delete them. The batch owner has no authority
/// over individual entries once created — entry ownership is fully independent.
///
/// Resolution order (getZoneConfig / getOwner):
///   1. If _entries[ordinal].owner != address(0)  → use entry
///   2. Otherwise                                 → use containing batch
///
/// Rate limiting
/// ─────────────
/// A token bucket (BUCKET_CAPACITY / REFILL_RATE / REFILL_PERIOD) smooths the
/// registration rate independently of block times, which vary on Rootstock
/// (typically 15–30s, inherited from Bitcoin merged mining).
/// The bucket refills continuously; BUCKET_CAPACITY bounds the burst,
/// REFILL_RATE bounds the sustained daily throughput.
///
/// At the default constants:
///   - Sustained max: 4,096 batch registrations/day (~12 trillion ordinals/day)
///   - Burst: 8 batch registrations before rate limiting kicks in
///   - Refill: one batch slot every ~21 seconds (well under one RSK block)
///
/// NS validation
/// ─────────────
/// The `ns` field is stored as a raw string and validated only for byte length
/// (max 255 bytes per RFC 1035). DNS label structure, character set, and wire
/// format are NOT validated on-chain. Off-chain resolvers should handle
/// malformed names gracefully.
contract MNSRegistry {
    // ─────────────────────────────────────────────────────────────────────────
    // Errors
    // ─────────────────────────────────────────────────────────────────────────

    /// @notice Registration is rate-limited. `estimatedWaitSeconds` is the time
    /// until enough tokens have accrued for one batch registration.
    error RateLimit(uint256 estimatedWaitSeconds);

    /// @notice The ordinal queried has not been registered.
    error OrdinalNotRegistered(uint64 ordinal);

    /// @notice The ordinal's batch has not been registered.
    error BatchNotRegistered(uint64 ordinal);

    // ─────────────────────────────────────────────────────────────────────────
    // Constants
    // ─────────────────────────────────────────────────────────────────────────

    /// @notice Maximum DNS name length per RFC 1035.
    /// Note: only byte length is enforced; label structure is not validated on-chain.
    uint256 public constant MAX_NS_LENGTH = 255;

    /// @notice Number of ordinals per batch. Batches cover [ordinal, ordinal + BATCH_SIZE).
    uint64 public constant BATCH_SIZE = 256;

    /// @notice Token bucket: ordinals (tokens) added per REFILL_PERIOD.
    /// At 1,048,576 ordinals/day the bucket refills at ~12 tokens/second.
    /// Each register() consumes BATCH_SIZE tokens.
    /// Sustained max: REFILL_RATE / BATCH_SIZE = 4,096 batch registrations/day.
    uint64 public constant REFILL_RATE = 2 ** 20;

    /// @notice Token bucket: maximum burst size.
    /// Burst limit: BUCKET_CAPACITY / BATCH_SIZE = 8 batch registrations before
    /// rate limiting kicks in. Sized to absorb a burst without meaningfully changing
    /// sustained throughput.
    uint64 public constant BUCKET_CAPACITY = 2048;

    /// @notice Period over which REFILL_RATE tokens are added to the bucket.
    uint256 public constant REFILL_PERIOD = 1 days;

    // ─────────────────────────────────────────────────────────────────────────
    // Data structures
    // ─────────────────────────────────────────────────────────────────────────

    /// @notice DNS zone configuration for an ordinal. Bundles the zone signing key
    /// (ZSK) with the zone's NS NSDNAME (https://datatracker.ietf.org/doc/html/rfc1035#section-3.3.11).
    /// The zone signing key commitment is a hash(keyType || pubkey) — analogous to a DNSSEC ZSK,
    /// but signs the entire DNS packet rather than individual RRsets.
    /// By hashing both the public key and its type, any signature scheme is supported
    /// off-chain without updating this contract.
    struct ZoneConfig {
        bytes32 zsk;
        string ns;
    }

    /// @notice A contiguous block of BATCH_SIZE ordinals with a single owner
    /// and default NS. Batches are append-only; ordinals increase
    /// monotonically by BATCH_SIZE.
    struct Batch {
        uint64 ordinal;
        address owner;
        ZoneConfig zone;
    }

    /// @notice A per-ordinal override. When present, supersedes the containing
    /// Batch for both owner and zone config resolution. Entries are permanent;
    /// they can be updated but not deleted.
    struct Entry {
        address owner;
        ZoneConfig zone;
    }

    /// @notice Input type for updateMany(). Groups all fields for a single
    /// entry update so callers pass an array of structs rather than parallel arrays.
    struct EntryUpdate {
        uint64 ordinal;
        address newOwner;
        bytes32 zsk;
        string ns;
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Storage
    // ─────────────────────────────────────────────────────────────────────────

    /// @dev Append-only array of batches, ordered by ascending ordinal.
    /// Invariant: _batches[i].ordinal == i * BATCH_SIZE. Enforced by register().
    Batch[] private _batches;

    /// @dev Per-ordinal entry overrides.
    mapping(uint64 ordinal => Entry) private _entries;

    // ─────────────────────────────────────────────────────────────────────────
    // Rate limiter state
    // ─────────────────────────────────────────────────────────────────────────

    /// @dev Current token bucket level.
    uint64 private _bucket = BUCKET_CAPACITY;

    /// @dev Timestamp of the last bucket write (consume or explicit update).
    /// Initialized to deployment time.
    uint256 private _lastRefill = block.timestamp;

    // ─────────────────────────────────────────────────────────────────────────
    // Events
    // ─────────────────────────────────────────────────────────────────────────

    /// @notice Emitted when a new Batch is registered.
    event BatchRegistered(uint64 indexed ordinal, address indexed owner, bytes32 zsk, string ns);

    /// @notice Emitted when a Batch's owner or NS is updated.
    event BatchUpdated(uint64 indexed ordinal, address indexed newOwner, bytes32 zsk, string ns);

    /// @notice Emitted when a new Entry is created (first time for an ordinal).
    event EntryCreated(uint64 indexed ordinal, address indexed newOwner, bytes32 zsk, string ns);

    /// @notice Emitted when an existing Entry is updated.
    event EntryUpdated(uint64 indexed ordinal, address indexed newOwner, bytes32 zsk, string ns);

    // ─────────────────────────────────────────────────────────────────────────
    // Views — registry state
    // ─────────────────────────────────────────────────────────────────────────

    /// @notice Returns the next ordinal that register() will assign.
    function nextOrdinal() external view returns (uint64) {
        return uint64(_batches.length * BATCH_SIZE);
    }

    /// @notice Resolves the effective ZoneConfig for a given ordinal.
    /// Entry takes precedence over Batch if one exists.
    /// @param target  The ordinal to resolve.
    /// @return The ZoneConfig (zone signing key + NS NSDNAME) for this ordinal.
    /// @custom:error OrdinalNotRegistered if the ordinal has not been registered.
    function getZoneConfig(uint64 target) external view returns (ZoneConfig memory) {
        Entry storage entry = _entries[target];
        if (entry.owner != address(0)) {
            return entry.zone;
        }
        (bool found, uint256 idx) = _getBatch(target);
        if (!found) revert OrdinalNotRegistered(target);
        return _batches[idx].zone;
    }

    /// @notice Resolves the effective owner for a given ordinal.
    /// Entry takes precedence over Batch if one exists.
    /// @param target  The ordinal to resolve.
    /// @return The owner address for the ordinal.
    /// @custom:error OrdinalNotRegistered if the ordinal has not been registered.
    function getOwner(uint64 target) external view returns (address) {
        return _getOwner(target);
    }

    /// @notice Returns the Batch that contains the given ordinal.
    /// @param ordinal  Any ordinal within the target batch.
    /// @return The Batch struct for the containing batch.
    /// @custom:error BatchNotRegistered if no batch covers this ordinal.
    function getBatch(uint64 ordinal) external view returns (Batch memory) {
        (bool found, uint256 idx) = _getBatch(ordinal);
        if (!found) revert BatchNotRegistered(ordinal);
        return _batches[idx];
    }

    /// @notice Returns whether a per-ordinal Entry exists for the given ordinal.
    /// Useful for resolvers that need to distinguish entry vs. batch-default resolution.
    /// @param ordinal  The ordinal to check.
    /// @return True if an Entry exists; false if resolution falls back to the batch.
    function hasEntry(uint64 ordinal) external view returns (bool) {
        return _entries[ordinal].owner != address(0);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Views — rate limiter
    // ─────────────────────────────────────────────────────────────────────────

    /// @notice Seconds until a batch registration is available (0 = now).
    function estimatedWaitTime() external view returns (uint256) {
        (uint64 current,,) = _computeBucket();
        if (current >= BATCH_SIZE) return 0;
        return _computeWait(current);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Mutating functions
    // ─────────────────────────────────────────────────────────────────────────

    /// @notice Register a new Batch. The caller becomes the owner.
    /// Permissionless — anyone may register, subject to rate limits.
    /// @param zsk  Zone signing key commitment: hash(keyType || pubkey).
    ///             Signs the entire DNS packet off-chain.
    /// @param ns   NS DNS name (max 255 bytes, byte length only). May be empty.
    /// @return r   The newly created Batch.
    function register(bytes32 zsk, string calldata ns) external returns (Batch memory) {
        _validateNs(bytes(ns).length);
        _consumeBucketToken();
        require(_batches.length < type(uint64).max / BATCH_SIZE, "ID space exhausted, what year is this?");
        uint64 newOrdinal = uint64(_batches.length * BATCH_SIZE);
        Batch memory batch = Batch(newOrdinal, msg.sender, ZoneConfig(zsk, ns));
        _batches.push(batch);
        emit BatchRegistered(newOrdinal, msg.sender, zsk, ns);
        return batch;
    }

    /// @notice Update the batch that contains the given ordinal.
    /// The caller must be the batch owner.
    /// @param ordinal   Any ordinal within the target batch.
    /// @param newOwner  New owner address. Must be non-zero.
    /// @param zsk       Zone signing key commitment. Pass bytes32(0) to clear.
    /// @param ns        NS DNS name (max 255 bytes). May be empty.
    function updateBatch(uint64 ordinal, address newOwner, bytes32 zsk, string calldata ns) external {
        (bool found, uint256 idx) = _getBatch(ordinal);
        if (!found) revert BatchNotRegistered(ordinal);
        require(_batches[idx].owner == msg.sender, "not owner");
        _validateNs(bytes(ns).length);
        _validateOwner(newOwner);
        uint64 batchOrdinal = _batches[idx].ordinal;
        _batches[idx].owner = newOwner;
        _batches[idx].zone.zsk = zsk;
        _batches[idx].zone.ns = ns;
        emit BatchUpdated(batchOrdinal, newOwner, zsk, ns);
    }

    /// @notice Create or update a per-ordinal Entry.
    /// Caller must be the current effective owner of the ordinal (entry owner
    /// if an entry exists, otherwise batch owner). Once created, only the
    /// entry owner can update it — the batch owner loses authority over this
    /// ordinal. This is intentional: entry ownership is fully independent of
    /// batch ownership.
    /// @param ordinal   The ordinal to update.
    /// @param newOwner  New entry owner. Must be non-zero.
    /// @param zsk       Zone signing key commitment. Pass bytes32(0) if unused.
    /// @param ns        NS DNS name (max 255 bytes). May be empty.
    /// @custom:error OrdinalNotRegistered if the ordinal's batch hasn't been registered.
    function update(uint64 ordinal, address newOwner, bytes32 zsk, string calldata ns) external {
        _update(ordinal, newOwner, zsk, ns);
    }

    /// @notice Create or update multiple per-ordinal Entries in a single transaction.
    /// Equivalent to calling update() N times but saves per-transaction base cost.
    /// Each update is independently authorized — the caller must be the effective
    /// owner of every ordinal in the array. If any update fails the entire call reverts.
    /// Registration (register()) is intentionally excluded; batch registration must
    /// remain a standalone call to preserve rate-limit fairness.
    /// @param updates  Array of EntryUpdate structs, each specifying ordinal, newOwner, zsk, ns.
    function updateMany(EntryUpdate[] calldata updates) external {
        for (uint256 i = 0; i < updates.length; i++) {
            _update(updates[i].ordinal, updates[i].newOwner, updates[i].zsk, updates[i].ns);
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Internal — mutate
    // ─────────────────────────────────────────────────────────────────────────

    /// @dev Shared logic for update() and updateMany().
    function _update(uint64 ordinal, address newOwner, bytes32 zsk, string memory ns) internal {
        _validateOwner(newOwner);
        _validateNs(bytes(ns).length);
        require(_getOwner(ordinal) == msg.sender, "not owner");
        bool existedBefore = _entries[ordinal].owner != address(0);
        Entry storage entry = _entries[ordinal];
        entry.owner = newOwner;
        entry.zone.zsk = zsk;
        entry.zone.ns = ns;
        if (existedBefore) {
            emit EntryUpdated(ordinal, newOwner, zsk, ns);
        } else {
            emit EntryCreated(ordinal, newOwner, zsk, ns);
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Internal — rate limiter
    // ─────────────────────────────────────────────────────────────────────────

    /// @dev Consumes BATCH_SIZE tokens from the bucket (one per ordinal in the batch).
    /// Reverts with RateLimit(estimatedWaitSeconds) if insufficient tokens.
    /// Advances _lastRefill by the floor of the accrual window so fractional time carries over.
    function _consumeBucketToken() private {
        (uint64 current,, uint256 accrued) = _computeBucket();
        if (current < BATCH_SIZE) {
            revert RateLimit(_computeWait(current));
        }
        _bucket = current - BATCH_SIZE;
        _lastRefill += (accrued * REFILL_PERIOD) / REFILL_RATE;
    }

    /// @dev Computes the wait time in seconds to accumulate `BATCH_SIZE - current` tokens.
    function _computeWait(uint64 current) private pure returns (uint256) {
        uint256 deficit = BATCH_SIZE - current;
        return (deficit * REFILL_PERIOD + REFILL_RATE - 1) / REFILL_RATE;
    }

    /// @dev Computes the current bucket level as of block.timestamp without
    /// writing state. Safe to call from view functions.
    function _computeBucket() private view returns (uint64 current, uint256 elapsed, uint256 accrued) {
        elapsed = block.timestamp - _lastRefill;
        accrued = (elapsed * REFILL_RATE) / REFILL_PERIOD;
        if (accrued == 0) {
            current = _bucket;
        } else {
            uint256 newBucket = uint256(_bucket) + accrued;
            current = newBucket > BUCKET_CAPACITY ? BUCKET_CAPACITY : uint64(newBucket);
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Internal — validation
    // ─────────────────────────────────────────────────────────────────────────

    function _validateOwner(address owner) private pure {
        require(owner != address(0), "invalid owner");
    }

    /// @dev Validates byte length only. DNS label structure is not checked.
    /// Accepts pre-computed length to avoid redundant bytes() conversion at call sites.
    function _validateNs(uint256 nsLength) private pure {
        require(nsLength <= MAX_NS_LENGTH, "ns too long");
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Internal — resolution
    // ─────────────────────────────────────────────────────────────────────────

    /// @dev Returns the effective owner of an ordinal (entry → batch fallback).
    /// Reverts if the ordinal has not been registered.
    function _getOwner(uint64 target) private view returns (address) {
        Entry storage entry = _entries[target];
        if (entry.owner != address(0)) {
            return entry.owner;
        }
        (bool found, uint256 idx) = _getBatch(target);
        if (!found) revert OrdinalNotRegistered(target);
        return _batches[idx].owner;
    }

    /// @dev Returns the index of the Batch that contains `target`.
    /// Batches are stored contiguously starting at ordinal 0 with step BATCH_SIZE,
    /// so the index is simply target / BATCH_SIZE.
    /// Returns (false, 0) if no batch exists for the target.
    function _getBatch(uint64 target) private view returns (bool found, uint256 idx) {
        idx = target / BATCH_SIZE;
        if (idx >= _batches.length) return (false, 0);
        return (true, idx);
    }
}
