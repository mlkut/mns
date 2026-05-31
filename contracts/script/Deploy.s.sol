pragma solidity ^0.8.28;

import "forge-std/Script.sol";
import "../src/MNSRegistrar.sol";

contract Deploy is Script {
    function run() public {
        vm.startBroadcast();
        new MNSRegistrar();
        vm.stopBroadcast();
    }
}
