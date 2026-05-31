// SPDX-License-Identifier: MIT
pragma solidity ^0.8.33;

// TODO:
// 1. Add basic key recovery with time window!

contract MNSRegistry {
    uint64 constant RANGE_SIZE = 256;
    // Max length of a DNS name (RFC 1035).
    uint256 constant MAX_NAMESERVER_LENGTH = 255;

    // Token bucket rate limiter: ~1,048,576 registrations per day, burst up to 512.
    uint64 constant REFILL_RATE = 2 ** 20;
    uint64 constant BUCKET_CAPACITY = 512;
    uint64 constant REFILL_PERIOD = 1 days;

    struct Range {
        uint64 ordinal;
        address owner;
        string nameServer;
    }

    struct Entry {
        address owner;
        string nameServer;
    }

    Range[] private _ranges;
    mapping(uint64 ordinal => Entry) private _entries;

    uint64 private _bucket = BUCKET_CAPACITY;
    uint256 private _lastRefill = block.timestamp;

    function next_ordinal() external view returns (uint64) {
        if (_ranges.length == 0) return 0;
        return _ranges[_ranges.length - 1].ordinal + RANGE_SIZE;
    }

    function getRange(uint256 index) external view returns (Range memory) {
        return _ranges[index];
    }

    function getEntry(uint64 ordinal) external view returns (Entry memory) {
        return _entries[ordinal];
    }

    function canRegister() external view returns (bool) {
        return _computeBucket() > 0;
    }

    function getNameServer(uint64 target) external view returns (string memory) {
        Entry storage entry = _entries[target];
        if (entry.owner != address(0)) {
            return entry.nameServer;
        }
        uint256 idx = _findRange(target);
        return _ranges[idx].nameServer;
    }

    function register(string calldata nameServer) external returns (Range memory) {
        _validateNameServer(nameServer);
        _consumeBucketToken();
        uint64 newOrdinal = _ranges.length == 0 ? 0 : _ranges[_ranges.length - 1].ordinal + RANGE_SIZE;
        Range memory r = Range(newOrdinal, msg.sender, nameServer);
        _ranges.push(r);
        return r;
    }

    function update(uint64 ordinal, address newOwner, string calldata newNameServer) external {
        _validateOwner(newOwner);
        _validateNameServer(newNameServer);
        require(_getOwner(ordinal) == msg.sender, "not owner");
        _entries[ordinal] = Entry(newOwner, newNameServer);
    }

    function updateRange(uint256 index, address newOwner, string calldata newNameServer) external {
        require(index < _ranges.length, "invalid range");
        require(_ranges[index].owner == msg.sender, "not owner");
        _validateOwner(newOwner);
        _validateNameServer(newNameServer);
        _ranges[index].owner = newOwner;
        _ranges[index].nameServer = newNameServer;
    }

    function _validateOwner(address owner) private pure {
        require(owner != address(0), "invalid owner");
    }

    function _consumeBucketToken() private {
        _bucket = _computeBucket();
        _lastRefill = block.timestamp;
        require(_bucket > 0, "rate limit: try later");
        _bucket--;
    }

    function _computeBucket() private view returns (uint64) {
        uint256 elapsed = block.timestamp - _lastRefill;
        uint64 accrued = uint64((elapsed * REFILL_RATE) / REFILL_PERIOD);
        if (accrued == 0) return _bucket;
        uint64 newBucket = _bucket + accrued;
        return newBucket > BUCKET_CAPACITY ? BUCKET_CAPACITY : newBucket;
    }

    function _validateNameServer(string calldata nameServer) private pure {
        require(bytes(nameServer).length <= MAX_NAMESERVER_LENGTH, "name server too long");
    }

    function _getOwner(uint64 target) private view returns (address) {
        Entry storage entry = _entries[target];
        if (entry.owner != address(0)) {
            return entry.owner;
        }
        uint256 idx = _findRange(target);
        return _ranges[idx].owner;
    }

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
