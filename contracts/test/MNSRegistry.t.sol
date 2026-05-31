pragma solidity ^0.8.33;

import "forge-std/Test.sol";
import "../src/MNSRegistry.sol";

contract MNSRegistryTest is Test {
    MNSRegistry registry;
    address alice = address(0xA11CE);
    address bob = address(0xB0B);

    function setUp() public {
        registry = new MNSRegistry();
    }

    function test_Register_ReturnsRangeStartingAtZero() public {
        MNSRegistry.Range memory r = registry.register(alice, "s1");
        assertEq(r.ordinal, 0);
        assertEq(r.owner, alice);
        assertEq(r.server, "s1");
    }

    function test_Register_SubsequentIncrementsBy256() public {
        MNSRegistry.Range memory r1 = registry.register(alice, "s1");
        MNSRegistry.Range memory r2 = registry.register(bob, "s2");
        assertEq(r1.ordinal, 0);
        assertEq(r2.ordinal, 256);
    }

    function test_Register_PushesToRangesArray() public {
        registry.register(alice, "s1");
        assertEq(registry.get_range(0).ordinal, 0);
    }

    function test_GetServer_ReturnsRangeServer() public {
        registry.register(alice, "s1");
        string memory server = registry.get_server(0);
        assertEq(server, "s1");
    }

    function test_GetServer_RoundsDownToCorrectRange() public {
        registry.register(alice, "s1");
        registry.register(bob, "s2");
        assertEq(registry.get_server(100), "s1");
        assertEq(registry.get_server(300), "s2");
    }

    function test_GetServer_ReturnsLastRangeForHighOrdinal() public {
        registry.register(alice, "s1");
        registry.register(bob, "s2");
        assertEq(registry.get_server(100000), "s2");
    }

    function test_GetServer_ReturnsEntryServer() public {
        registry.register(alice, "s1");
        vm.prank(alice);
        registry.update(50, bob, "overridden");
        assertEq(registry.get_server(50), "overridden");
    }

    function test_GetServer_ReturnsRangeServerWhenNoEntry() public {
        registry.register(alice, "s1");
        vm.prank(alice);
        registry.update(50, bob, "overridden");
        assertEq(registry.get_server(51), "s1");
    }

    function test_GetServer_BoundaryExactMatch() public {
        registry.register(alice, "s1");
        registry.register(bob, "s2");
        assertEq(registry.get_server(256), "s2");
    }

    function test_Entry_SetsEntryForOrdinal() public {
        registry.register(alice, "s1");
        vm.prank(alice);
        registry.update(50, bob, "custom");
        MNSRegistry.Entry memory e = registry.get_entry(50);
        assertEq(e.owner, bob);
        assertEq(e.server, "custom");
    }

    function test_Update_RangeOwnerCreatesEntry() public {
        registry.register(alice, "s1");
        vm.prank(alice);
        registry.update(50, bob, "ok");
        assertEq(registry.get_entry(50).owner, bob);
    }

    function test_Update_RevertsWhenNotOwnerNoEntry() public {
        registry.register(alice, "s1");
        vm.prank(bob);
        vm.expectRevert("not owner");
        registry.update(50, bob, "");
    }

    function test_Update_EntryOwnerCanUpdate() public {
        registry.register(alice, "s1");
        vm.prank(alice);
        registry.update(50, bob, "v1");
        vm.prank(bob);
        registry.update(50, alice, "v2");
        assertEq(registry.get_entry(50).owner, alice);
    }

    function test_Update_RevertsWhenNotOwnerExistingEntry() public {
        registry.register(alice, "s1");
        vm.prank(alice);
        registry.update(50, bob, "v1");
        vm.prank(alice);
        vm.expectRevert("not owner");
        registry.update(50, bob, "v2");
    }

    function test_Register_CanPushMultiple() public {
        for (uint256 i = 0; i < 5; i++) {
            registry.register(alice, "s");
        }
        assertEq(registry.last_ordinal(), 1279);
    }

    function test_Register_ReturnedRangeMatchesStorage() public {
        MNSRegistry.Range memory r = registry.register(alice, "s1");
        MNSRegistry.Range memory stored = registry.get_range(0);
        assertEq(r.ordinal, stored.ordinal);
        assertEq(r.owner, stored.owner);
        assertEq(r.server, stored.server);
    }
}
