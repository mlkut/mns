// SPDX-License-Identifier: MIT
pragma solidity ^0.8.33;

/// @title MNSRegistry
/// @notice Decentralized name registry on Rootstock.
///
/// Architecture
/// ────────────
/// Ordinal space is divided into fixed-size Ranges of RANGE_SIZE (256) ordinals each.
/// Anyone may register a new Range (consuming a rate-limit token). Each Range has an
/// owner and a default nameserver that resolves for every ordinal in [ordinal, ordinal+255].
///
/// Individual ordinals can be overridden via Entries. An Entry takes precedence over its
/// containing Range for both owner and nameserver resolution. Entries are permanent once
/// created; the entry owner may update but not delete them. The range owner has no authority
/// over individual entries once created — entry ownership is fully independent.
///
/// Resolution order (getNameserverConfig / getOwner):
///   1. If _entries[ordinal].owner != address(0)  → use entry
///   2. Otherwise                                 → use containing range
///
/// Rate limiting
/// ─────────────
/// Two independent limits are layered:
///   • Token bucket (BUCKET_CAPACITY / REFILL_RATE / REFILL_PERIOD): smooths the daily
///     registration rate independently of block times, which vary on Rootstock. The bucket
///     refills continuously; BUCKET_CAPACITY bounds the burst, REFILL_RATE bounds the
///     sustained daily throughput.
///   • Per-block cap (MAX_PER_BLOCK): prevents a single block from exhausting the daily
///     limit by many transactions. Resets every block.
contract MNSRegistry {
    // ─────────────────────────────────────────────────────────────────────────
    // Constants
    // ─────────────────────────────────────────────────────────────────────────

    /// @notice Number of ordinals per range. Ranges cover [ordinal, ordinal + RANGE_SIZE).
    uint64 public constant RANGE_SIZE = 256;

    /// @notice Maximum DNS name length per RFC 1035.
    uint256 public constant MAX_NAMESERVER_LENGTH = 255;

    /// @notice Maximum registrations allowed in a single block.
    /// Prevents a single block from being exhausted across many callers.
    uint64 public constant MAX_PER_BLOCK = 10;

    /// @notice Token bucket: sustained registrations added per REFILL_PERIOD.
    /// At 1,048,576 / day the bucket refills at ~12 tokens/second.
    uint64 public constant REFILL_RATE = 2 ** 20;

    /// @notice Token bucket: maximum burst size. Once depleted, callers must
    /// wait for the bucket to refill. Should be >= MAX_PER_BLOCK so that a
    /// freshly-deployed contract isn't immediately block-rate-limited.
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

    /// @notice A contiguous block of RANGE_SIZE ordinals with a single owner
    /// and default nameserver. Ranges are append-only; ordinals increase
    /// monotonically by RANGE_SIZE.
    struct Range {
        uint64 ordinal;
        address owner;
        NameserverConfig ns;
    }

    /// @notice A per-ordinal override. When present, supersedes the containing
    /// Range for both owner and nameserver resolution. Entries are permanent;
    /// they can be updated but not deleted. Use tombstoning (point nameServer
    /// to a well-known sentinel string) if you need to signal decommission.
    struct Entry {
        address owner;
        NameserverConfig ns;
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Storage
    // ─────────────────────────────────────────────────────────────────────────

    /// @dev Append-only array of ranges, ordered by ascending ordinal.
    Range[] private _ranges;

    /// @dev Per-ordinal entry overrides.
    mapping(uint64 ordinal => Entry) private _entries;

    // ─────────────────────────────────────────────────────────────────────────
    // Rate limiter state
    // ─────────────────────────────────────────────────────────────────────────

    /// @dev Current token bucket level.
    uint64 private _bucket = BUCKET_CAPACITY;

    /// @dev Timestamp of the last bucket write (consume or explicit update).
    uint256 private _lastRefill = block.timestamp;

    /// @dev Number of registrations in the current block (resets each block).
    uint64 private _blockCount;

    /// @dev Block number when _blockCount was last reset.
    uint256 private _lastBlock;

    // ─────────────────────────────────────────────────────────────────────────
    // Events
    // ─────────────────────────────────────────────────────────────────────────

    /// @notice Emitted when a new Range is registered.
    event RangeRegistered(uint64 indexed ordinal, address indexed owner, string nameServer, bytes32 signerHash);

    /// @notice Emitted when a Range's owner or nameserver is updated.
    event RangeUpdated(
        uint256 indexed index, uint64 indexed ordinal, address indexed newOwner, string nameServer, bytes32 signerHash
    );

    /// @notice Emitted when a new Entry is created (first time for an ordinal).
    event EntryCreated(uint64 indexed ordinal, address indexed newOwner, string nameServer, bytes32 signerHash);

    /// @notice Emitted when an existing Entry is updated.
    event EntryUpdated(uint64 indexed ordinal, address indexed newOwner, string nameServer, bytes32 signerHash);

    // ─────────────────────────────────────────────────────────────────────────
    // Views — registry state
    // ─────────────────────────────────────────────────────────────────────────

    /// @notice Returns the ordinal that the next registered range will receive.
    /// Returns 0 if no ranges exist yet.
    function nextOrdinal() external view returns (uint64) {
        if (_ranges.length == 0) return 0;
        return _ranges[_ranges.length - 1].ordinal + RANGE_SIZE;
    }

    /// @notice Total number of registered ranges.
    function rangeCount() external view returns (uint256) {
        return _ranges.length;
    }

    /// @notice Returns the Range at the given index.
    function getRange(uint256 index) external view returns (Range memory) {
        require(index < _ranges.length, "index out of bounds");
        return _ranges[index];
    }

    /// @notice Returns the Entry for the given ordinal.
    /// Returns a zero-value Entry if none exists (check owner != address(0)).
    function getEntry(uint64 ordinal) external view returns (Entry memory) {
        return _entries[ordinal];
    }

    /// @notice Resolves the effective nameserver config for a given ordinal.
    /// Entry takes precedence over Range if one exists.
    function getNameserverConfig(uint64 target) external view returns (NameserverConfig memory) {
        Entry storage entry = _entries[target];
        if (entry.owner != address(0)) {
            return entry.ns;
        }
        uint256 idx = _findRange(target);
        return _ranges[idx].ns;
    }

    /// @notice Resolves the effective owner for a given ordinal.
    /// Entry takes precedence over Range if one exists.
    function getOwner(uint64 target) external view returns (address) {
        return _getOwner(target);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Views — rate limiter
    // ─────────────────────────────────────────────────────────────────────────

    /// @notice Returns true if at least one registration is possible right now.
    /// Accounts for both the token bucket and the per-block cap.
    /// @dev The block cap resets at the next block, so a false result from the
    /// block cap becomes true as soon as a new block is mined.
    function canRegister() external view returns (bool) {
        if (_computeBucket() == 0) return false;
        if (block.number == _lastBlock && _blockCount >= MAX_PER_BLOCK) return false;
        return true;
    }

    /// @notice Current effective token bucket level (read-only, not persisted).
    function bucketLevel() external view returns (uint64) {
        return _computeBucket();
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Mutating functions
    // ─────────────────────────────────────────────────────────────────────────

    /// @notice Register a new Range. The caller becomes the owner.
    /// Permissionless — anyone may register, subject to rate limits.
    /// @param nameServer  Default nameserver for all ordinals in this range.
    /// @param signerHash  Hash of the signer's public key (and type). Pass
    ///                    bytes32(0) if no off-chain signing key is configured.
    /// @return r The newly created Range.
    function register(string calldata nameServer, bytes32 signerHash) external returns (Range memory r) {
        _validateNameServer(nameServer);
        _consumeBucketToken();
        uint64 newOrdinal = _ranges.length == 0 ? 0 : _ranges[_ranges.length - 1].ordinal + RANGE_SIZE;
        r = Range(newOrdinal, msg.sender, NameserverConfig(nameServer, signerHash));
        _ranges.push(r);
        emit RangeRegistered(newOrdinal, msg.sender, nameServer, signerHash);
    }

    /// @notice Update the range that contains the given ordinal.
    /// The caller must be the range owner.
    /// @param ordinal       Any ordinal within the target range.
    /// @param newOwner      New owner address. Must be non-zero.
    /// @param newNameServer New default nameserver for the range.
    /// @param signerHash    Hash of the signer's public key (and type). Pass
    ///                      bytes32(0) to clear the signing key.
    function updateRange(uint64 ordinal, address newOwner, string calldata newNameServer, bytes32 signerHash)
        external
    {
        uint256 idx = _findRange(ordinal);
        require(_ranges[idx].owner == msg.sender, "not owner");
        _validateOwner(newOwner);
        _validateNameServer(newNameServer);
        _ranges[idx].owner = newOwner;
        _ranges[idx].ns.nameServer = newNameServer;
        _ranges[idx].ns.signerHash = signerHash;
        emit RangeUpdated(idx, _ranges[idx].ordinal, newOwner, newNameServer, signerHash);
    }

    /// @notice Create or update a per-ordinal Entry.
    /// Caller must be the current effective owner of the ordinal (entry owner
    /// if an entry exists, otherwise range owner). Once created, only the
    /// entry owner can update it — the range owner loses authority over this
    /// ordinal. This is intentional: entry ownership is fully independent of
    /// range ownership.
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
            emit EntryCreated(ordinal, newOwner, newNameServer, signerHash);
        } else {
            emit EntryUpdated(ordinal, newOwner, newNameServer, signerHash);
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Internal — rate limiter
    // ─────────────────────────────────────────────────────────────────────────

    /// @dev Consumes one token from the bucket and increments the block counter.
    /// Reverts if either limit is exhausted.
    function _consumeBucketToken() private {
        // Per-block limit: reset counter on new block.
        if (block.number != _lastBlock) {
            _lastBlock = block.number;
            _blockCount = 0;
        }
        require(_blockCount < MAX_PER_BLOCK, "rate limit: block cap");
        _blockCount++;

        // Token bucket limit.
        uint64 current = _computeBucket();
        require(current > 0, "rate limit: daily cap");
        _bucket = current - 1;
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

    /// @dev Returns the effective owner of an ordinal (entry → range fallback).
    function _getOwner(uint64 target) private view returns (address) {
        Entry storage entry = _entries[target];
        if (entry.owner != address(0)) {
            return entry.owner;
        }
        uint256 idx = _findRange(target);
        return _ranges[idx].owner;
    }

    /// @dev Binary search: returns the index of the Range that contains `target`.
    /// Reverts if no ranges exist or if target falls outside all ranges.
    ///
    /// Invariant: ordinals always start at 0 and increment by RANGE_SIZE, so
    /// the left == 0 underflow case (target less than all range ordinals) is
    /// only reachable if _ranges is empty, which is guarded above.
    function _findRange(uint64 target) private view returns (uint256) {
        require(_ranges.length > 0, "no ranges");
        uint256 left;
        uint256 right = _ranges.length;
        while (left < right) {
            uint256 mid = (left + right) / 2;
            if (_ranges[mid].ordinal <= target) {
                left = mid + 1;
            } else {
                right = mid;
            }
        }
        uint256 rangeIdx = left - 1;
        require(target < _ranges[rangeIdx].ordinal + RANGE_SIZE, "ordinal out of range");
        return rangeIdx;
    }
}
