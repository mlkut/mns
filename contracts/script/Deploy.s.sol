pragma solidity ^0.8.33;

import "forge-std/Script.sol";
import "../src/MNSRegistry.sol";

contract Deploy is Script {
    function run() public {
        vm.startBroadcast();
        new MNSRegistry();
        vm.stopBroadcast();
    }
}
