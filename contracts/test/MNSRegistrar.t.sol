pragma solidity ^0.8.28;

import "forge-std/Test.sol";
import "../src/MNSRegistrar.sol";

contract MNSRegistrarTest is Test {
    MNSRegistrar registrar;

    function setUp() public {
        registrar = new MNSRegistrar();
    }

    function test_Register() public {
        registrar.register("hello");
        bytes32 hash = keccak256("hello");
        assertEq(registrar.ownerOf(hash), address(this));
    }

    function test_RevertWhen_DuplicateRegister() public {
        registrar.register("hello");
        vm.expectRevert("already registered");
        registrar.register("hello");
    }

    function test_Transfer() public {
        registrar.register("hello");
        bytes32 hash = keccak256("hello");
        registrar.transfer(hash, address(0x123));
        assertEq(registrar.ownerOf(hash), address(0x123));
    }

    function test_RevertWhen_UnauthorizedTransfer() public {
        registrar.register("hello");
        bytes32 hash = keccak256("hello");
        vm.prank(address(0x999));
        vm.expectRevert("not owner");
        registrar.transfer(hash, address(0x123));
    }
}
