pragma solidity ^0.8.33;

import {Script} from "forge-std/Script.sol";
import {MNSRegistry} from "../src/MNSRegistry.sol";

contract Deploy is Script {
    function run() public {
        vm.startBroadcast();
        new MNSRegistry();
        vm.stopBroadcast();
    }
}
