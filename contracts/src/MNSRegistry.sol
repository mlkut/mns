pragma solidity ^0.8.33;

contract MNSRegistry {
    mapping(bytes32 namehash => address owner) public records;
    mapping(bytes32 namehash => string) public names;

    event Registered(bytes32 indexed namehash, string name, address indexed owner);
    event Transferred(bytes32 indexed namehash, address indexed from, address indexed to);

    function register(string calldata name) external {
        bytes32 hash = keccak256(abi.encodePacked(name));
        require(records[hash] == address(0), "already registered");
        records[hash] = msg.sender;
        names[hash] = name;
        emit Registered(hash, name, msg.sender);
    }

    function transfer(bytes32 namehash, address to) external {
        require(records[namehash] == msg.sender, "not owner");
        records[namehash] = to;
        emit Transferred(namehash, msg.sender, to);
    }

    function ownerOf(bytes32 namehash) external view returns (address) {
        return records[namehash];
    }
}
