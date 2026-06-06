// SPDX-License-Identifier: MIT
pragma solidity ^0.8.33;

/// @title MNSRegistry
/// @notice Decentralized name registry on Rootstock.
///
/// Architecture
/// ────────────
/// Ordinal space is divided into fixed-size Batches of BATCH_SIZE (256) ordinals each.
/// Anyone may register a new Batch (consuming a rate-limit token). Each Batch has an
/// owner and a default nameserver that resolves for every ordinal in [ordinal, ordinal+255].
///
/// Individual ordinals can be overridden via Entries. An Entry takes precedence over its
/// containing Batch for both owner and nameserver resolution. Entries are permanent once
/// created; the entry owner may update but not delete them. The batch owner has no authority
/// over individual entries once created — entry ownership is fully independent.
///
/// Resolution order (getNameserverConfig / getOwner):
///   1. If _entries[ordinal].owner != address(0)  → use entry
///   2. Otherwise                                 → use containing batch
///
/// Rate limiting
/// ─────────────
/// A token bucket (BUCKET_CAPACITY / REFILL_RATE / REFILL_PERIOD) smooths the
/// registration rate independently of block times, which vary on Rootstock. The
/// bucket refills continuously; BUCKET_CAPACITY bounds the burst, REFILL_RATE
/// bounds the sustained daily throughput.
contract MNSRegistry {
    // ─────────────────────────────────────────────────────────────────────────
    // Constants
    // ─────────────────────────────────────────────────────────────────────────

    /// @notice Number of ordinals per batch. Batches cover [ordinal, ordinal + BATCH_SIZE).
    uint64 public constant BATCH_SIZE = 256;

    /// @notice Maximum DNS name length per RFC 1035.
    uint256 public constant MAX_NAMESERVER_LENGTH = 255;

    /// @notice Token bucket: ordinals (tokens) added per REFILL_PERIOD.
    /// At 1,048,576 ordinals/day the bucket refills at ~12 tokens/second.
    /// Each register() consumes BATCH_SIZE tokens.
    uint64 public constant REFILL_RATE = 2 ** 20;

    /// @notice Token bucket: maximum burst size. Once depleted, callers must
    /// wait for the bucket to refill.
    uint64 public constant BUCKET_CAPACITY = 512;

    /// @notice Period over which REFILL_RATE tokens are added to the bucket.
    uint256 public constant REFILL_PERIOD = 1 days;

    // ─────────────────────────────────────────────────────────────────────────
    // Data structures
    // ─────────────────────────────────────────────────────────────────────────

    /// @notice Bundles a nameserver with the hash of its signing key.
    /// The signer hash is keccak256(abi.encode(pubkey, keyType)) — by hashing
    /// both the public key and its type, any signature scheme is supported
    /// off-chain without updating this contract.
    struct NameserverConfig {
        string nameServer;
        bytes32 signerHash;
    }

    /// @notice A contiguous block of BATCH_SIZE ordinals with a single owner
    /// and default nameserver. Batches are append-only; ordinals increase
    /// monotonically by BATCH_SIZE.
    struct Batch {
        uint64 ordinal;
        address owner;
        NameserverConfig ns;
    }

    /// @notice A per-ordinal override. When present, supersedes the containing
    /// Batch for both owner and nameserver resolution. Entries are permanent;
    /// they can be updated but not deleted. Use tombstoning (point nameServer
    /// to a well-known sentinel string) if you need to signal decommission.
    struct Entry {
        address owner;
        NameserverConfig ns;
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Storage
    // ─────────────────────────────────────────────────────────────────────────

    /// @dev Append-only array of batches, ordered by ascending ordinal.
    Batch[] private _batches;

    /// @dev Per-ordinal entry overrides.
    mapping(uint64 ordinal => Entry) private _entries;

    // ─────────────────────────────────────────────────────────────────────────
    // Rate limiter state
    // ─────────────────────────────────────────────────────────────────────────

    /// @dev Current token bucket level.
    uint64 private _bucket = BUCKET_CAPACITY;

    /// @dev Timestamp of the last bucket write (consume or explicit update).
    uint256 private _lastRefill = block.timestamp;

    // ─────────────────────────────────────────────────────────────────────────
    // Events
    // ─────────────────────────────────────────────────────────────────────────

    /// @notice Emitted when a new Batch is registered.
    event BatchRegistered(uint64 indexed ordinal, address indexed owner, string nameServer, bytes32 signerHash);

    /// @notice Emitted when a Batch's owner or nameserver is updated.
    event BatchUpdated(
        uint256 indexed index, uint64 indexed ordinal, address indexed newOwner, string nameServer, bytes32 signerHash
    );

    /// @notice Emitted when a new Entry is created (first time for an ordinal).
    event EntryCreated(uint64 indexed ordinal, address indexed newOwner, string nameServer, bytes32 signerHash);

    /// @notice Emitted when an existing Entry is updated.
    event EntryUpdated(uint64 indexed ordinal, address indexed newOwner, string nameServer, bytes32 signerHash);

    // ─────────────────────────────────────────────────────────────────────────
    // Views — registry state
    // ─────────────────────────────────────────────────────────────────────────

    /// @notice Returns the ordinal that the next registered batch will receive.
    /// Returns 0 if no batches exist yet.
    function nextOrdinal() external view returns (uint64) {
        if (_batches.length == 0) return 0;
        return _batches[_batches.length - 1].ordinal + BATCH_SIZE;
    }

    /// @notice Total number of registered batches.
    function batchCount() external view returns (uint256) {
        return _batches.length;
    }

    /// @notice Returns the Batch at the given index.
    function getBatch(uint256 index) external view returns (Batch memory) {
        require(index < _batches.length, "index out of bounds");
        return _batches[index];
    }

    /// @notice Returns the Entry for the given ordinal.
    /// Returns a zero-value Entry if none exists (check owner != address(0)).
    function getEntry(uint64 ordinal) external view returns (Entry memory) {
        return _entries[ordinal];
    }

    /// @notice Resolves the effective nameserver config for a given ordinal.
    /// Entry takes precedence over Batch if one exists.
    function getNameserverConfig(uint64 target) external view returns (NameserverConfig memory) {
        Entry storage entry = _entries[target];
        if (entry.owner != address(0)) {
            return entry.ns;
        }
        uint256 idx = _findBatch(target);
        return _batches[idx].ns;
    }

    /// @notice Resolves the effective owner for a given ordinal.
    /// Entry takes precedence over Batch if one exists.
    function getOwner(uint64 target) external view returns (address) {
        return _getOwner(target);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Views — rate limiter
    // ─────────────────────────────────────────────────────────────────────────

    /// @notice Returns true if at least one registration is possible right now.
    function canRegister() external view returns (bool) {
        return _computeBucket() >= BATCH_SIZE;
    }

    /// @notice Current effective token bucket level (read-only, not persisted).
    function bucketLevel() external view returns (uint64) {
        return _computeBucket();
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Mutating functions
    // ─────────────────────────────────────────────────────────────────────────

    /// @notice Register a new Batch. The caller becomes the owner.
    /// Permissionless — anyone may register, subject to rate limits.
    /// @param nameServer  Default nameserver for all ordinals in this batch.
    /// @param signerHash  Hash of the signer's public key (and type). Pass
    ///                    bytes32(0) if no off-chain signing key is configured.
    /// @return r The newly created Batch.
    function register(string calldata nameServer, bytes32 signerHash) external returns (Batch memory r) {
        _validateNameServer(nameServer);
        _consumeBucketToken();
        uint64 newOrdinal = _batches.length == 0 ? 0 : _batches[_batches.length - 1].ordinal + BATCH_SIZE;
        r = Batch(newOrdinal, msg.sender, NameserverConfig(nameServer, signerHash));
        _batches.push(r);
        emit BatchRegistered(newOrdinal, msg.sender, nameServer, signerHash);
    }

    /// @notice Update the batch that contains the given ordinal.
    /// The caller must be the batch owner.
    /// @param ordinal       Any ordinal within the target batch.
    /// @param newOwner      New owner address. Must be non-zero.
    /// @param newNameServer New default nameserver for the batch.
    /// @param signerHash    Hash of the signer's public key (and type). Pass
    ///                      bytes32(0) to clear the signing key.
    function updateBatch(uint64 ordinal, address newOwner, string calldata newNameServer, bytes32 signerHash)
        external
    {
        uint256 idx = _findBatch(ordinal);
        require(_batches[idx].owner == msg.sender, "not owner");
        _validateOwner(newOwner);
        _validateNameServer(newNameServer);
        _batches[idx].owner = newOwner;
        _batches[idx].ns.nameServer = newNameServer;
        _batches[idx].ns.signerHash = signerHash;
        emit BatchUpdated(idx, _batches[idx].ordinal, newOwner, newNameServer, signerHash);
    }

    /// @notice Create or update a per-ordinal Entry.
    /// Caller must be the current effective owner of the ordinal (entry owner
    /// if an entry exists, otherwise batch owner). Once created, only the
    /// entry owner can update it — the batch owner loses authority over this
    /// ordinal. This is intentional: entry ownership is fully independent of
    /// batch ownership.
    /// @param ordinal       The ordinal to update.
    /// @param newOwner      New entry owner. Must be non-zero.
    /// @param newNameServer Nameserver for this specific ordinal.
    /// @param signerHash    Hash of the signer's public key (and type) that
    ///                      signs records off-chain. Pass bytes32(0) if unused.
    function update(uint64 ordinal, address newOwner, string calldata newNameServer, bytes32 signerHash) external {
        _validateOwner(newOwner);
        _validateNameServer(newNameServer);
        require(_getOwner(ordinal) == msg.sender, "not owner");
        bool existedBefore = _entries[ordinal].owner != address(0);
        _entries[ordinal] = Entry(newOwner, NameserverConfig(newNameServer, signerHash));
        if (existedBefore) {
            emit EntryUpdated(ordinal, newOwner, newNameServer, signerHash);
        } else {
            emit EntryCreated(ordinal, newOwner, newNameServer, signerHash);
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Internal — rate limiter
    // ─────────────────────────────────────────────────────────────────────────

    /// @dev Consumes BATCH_SIZE tokens from the bucket (one per ordinal in the batch).
    /// Reverts if the bucket doesn't have enough tokens.
    function _consumeBucketToken() private {
        uint64 current = _computeBucket();
        require(current >= BATCH_SIZE, "rate limit");
        _bucket = current - BATCH_SIZE;
        _lastRefill = block.timestamp;
    }

    /// @dev Computes the current bucket level as of block.timestamp without
    /// writing state. Safe to call from view functions.
    function _computeBucket() private view returns (uint64) {
        uint256 elapsed = block.timestamp - _lastRefill;
        uint256 accrued = (elapsed * REFILL_RATE) / REFILL_PERIOD;
        if (accrued == 0) return _bucket;
        uint256 newBucket = uint256(_bucket) + accrued;
        return newBucket > BUCKET_CAPACITY ? BUCKET_CAPACITY : uint64(newBucket);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Internal — validation
    // ─────────────────────────────────────────────────────────────────────────

    function _validateOwner(address owner) private pure {
        require(owner != address(0), "invalid owner");
    }

    function _validateNameServer(string calldata nameServer) private pure {
        require(bytes(nameServer).length > 0, "empty nameserver");
        require(bytes(nameServer).length <= MAX_NAMESERVER_LENGTH, "nameserver too long");
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Internal — resolution
    // ─────────────────────────────────────────────────────────────────────────

    /// @dev Returns the effective owner of an ordinal (entry → batch fallback).
    function _getOwner(uint64 target) private view returns (address) {
        Entry storage entry = _entries[target];
        if (entry.owner != address(0)) {
            return entry.owner;
        }
        uint256 idx = _findBatch(target);
        return _batches[idx].owner;
    }

    /// @dev Binary search: returns the index of the Batch that contains `target`.
    /// Reverts if no batches exist or if target falls outside all batches.
    ///
    /// Invariant: ordinals always start at 0 and increment by BATCH_SIZE, so
    /// the left == 0 underflow case (target less than all batch ordinals) is
    /// only reachable if _batches is empty, which is guarded above.
    function _findBatch(uint64 target) private view returns (uint256) {
        require(_batches.length > 0, "no batches");
        uint256 left;
        uint256 right = _batches.length;
        while (left < right) {
            uint256 mid = (left + right) / 2;
            if (_batches[mid].ordinal <= target) {
                left = mid + 1;
            } else {
                right = mid;
            }
        }
        uint256 batchIdx = left - 1;
        require(target < _batches[batchIdx].ordinal + BATCH_SIZE, "ordinal out of batch");
        return batchIdx;
    }
}
