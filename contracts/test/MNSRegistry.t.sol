pragma solidity ^0.8.33;

import "forge-std/Test.sol";
import "../src/MNSRegistry.sol";

contract MNSRegistryTest is Test {
    MNSRegistry registry;

    function setUp() public {
        registry = new MNSRegistry();
    }

    function test_Register() public {
        registry.register("hello");
        bytes32 hash = keccak256("hello");
        assertEq(registry.ownerOf(hash), address(this));
    }

    function test_RevertWhen_DuplicateRegister() public {
        registry.register("hello");
        vm.expectRevert("already registered");
        registry.register("hello");
    }

    function test_Transfer() public {
        registry.register("hello");
        bytes32 hash = keccak256("hello");
        registry.transfer(hash, address(0x123));
        assertEq(registry.ownerOf(hash), address(0x123));
    }

    function test_RevertWhen_UnauthorizedTransfer() public {
        registry.register("hello");
        bytes32 hash = keccak256("hello");
        vm.prank(address(0x999));
        vm.expectRevert("not owner");
        registry.transfer(hash, address(0x123));
    }
}
