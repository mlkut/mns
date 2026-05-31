pragma solidity ^0.8.33;

contract MNSRegistry {
    uint64 constant RANGE_SIZE = 256;

    struct Range {
        uint64 ordinal;
        address owner;
        string server;
    }

    struct Entry {
        address owner;
        string server;
    }

    Range[] private _ranges;
    mapping(uint64 ordinal => Entry) private _entries;

    function last_ordinal() external view returns (uint64) {
        if (_ranges.length == 0) return 0;
        return _ranges[_ranges.length - 1].ordinal + RANGE_SIZE - 1;
    }

    function get_range(uint256 index) external view returns (Range memory) {
        return _ranges[index];
    }

    function get_entry(uint64 ordinal) external view returns (Entry memory) {
        return _entries[ordinal];
    }

    function get_server(uint64 target) external view returns (string memory) {
        Entry storage entry = _entries[target];
        if (entry.owner != address(0)) {
            return entry.server;
        }
        uint256 idx = _findRange(target);
        return _ranges[idx].server;
    }

    function register(address owner, string calldata server) external returns (Range memory) {
        uint64 newOrdinal = _ranges.length == 0 ? 0 : _ranges[_ranges.length - 1].ordinal + RANGE_SIZE;
        Range memory r = Range(newOrdinal, owner, server);
        _ranges.push(r);
        return r;
    }

    function update(uint64 ordinal, address newOwner, string calldata newServer) external {
        require(_getOwner(ordinal) == msg.sender, "not owner");
        _entries[ordinal] = Entry(newOwner, newServer);
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
        return left - 1;
    }
}
