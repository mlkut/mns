// SPDX-License-Identifier: MIT
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
        vm.prank(alice);
        MNSRegistry.Range memory r = registry.register("s1");
        assertEq(r.ordinal, 0);
        assertEq(r.owner, alice);
        assertEq(r.nameServer, "s1");
    }

    function test_Register_SubsequentIncrementsBy256() public {
        vm.prank(alice);
        MNSRegistry.Range memory r1 = registry.register("s1");
        vm.prank(bob);
        MNSRegistry.Range memory r2 = registry.register("s2");
        assertEq(r1.ordinal, 0);
        assertEq(r2.ordinal, 256);
    }

    function test_Register_PushesToRangesArray() public {
        vm.prank(alice);
        registry.register("s1");
        assertEq(registry.getRange(0).ordinal, 0);
    }

    function test_GetNameServer_ReturnsRangeNameServer() public {
        vm.prank(alice);
        registry.register("s1");
        string memory nameServer = registry.getNameServer(0);
        assertEq(nameServer, "s1");
    }

    function test_GetNameServer_RoundsDownToCorrectRange() public {
        vm.prank(alice);
        registry.register("s1");
        vm.prank(bob);
        registry.register("s2");
        assertEq(registry.getNameServer(100), "s1");
        assertEq(registry.getNameServer(300), "s2");
    }

    function test_GetNameServer_ReturnsLastRangeForHighOrdinal() public {
        vm.prank(alice);
        registry.register("s1");
        vm.prank(bob);
        registry.register("s2");
        assertEq(registry.getNameServer(100000), "s2");
    }

    function test_GetNameServer_ReturnsEntryNameServer() public {
        vm.prank(alice);
        registry.register("s1");
        vm.prank(alice);
        registry.update(50, bob, "overridden");
        assertEq(registry.getNameServer(50), "overridden");
    }

    function test_GetNameServer_ReturnsRangeNameServerWhenNoEntry() public {
        vm.prank(alice);
        registry.register("s1");
        vm.prank(alice);
        registry.update(50, bob, "overridden");
        assertEq(registry.getNameServer(51), "s1");
    }

    function test_GetNameServer_BoundaryExactMatch() public {
        vm.prank(alice);
        registry.register("s1");
        vm.prank(bob);
        registry.register("s2");
        assertEq(registry.getNameServer(256), "s2");
    }

    function test_Entry_SetsEntryNameServerForOrdinal() public {
        vm.prank(alice);
        registry.register("s1");
        vm.prank(alice);
        registry.update(50, bob, "custom");
        MNSRegistry.Entry memory e = registry.getEntry(50);
        assertEq(e.owner, bob);
        assertEq(e.nameServer, "custom");
    }

    function test_Update_RangeOwnerCreatesEntry() public {
        vm.prank(alice);
        registry.register("s1");
        vm.prank(alice);
        registry.update(50, bob, "ok");
        assertEq(registry.getEntry(50).owner, bob);
    }

    function test_Update_RevertsWhenNotOwnerNoEntry() public {
        vm.prank(alice);
        registry.register("s1");
        vm.prank(bob);
        vm.expectRevert("not owner");
        registry.update(50, bob, "");
    }

    function test_Update_EntryOwnerCanUpdate() public {
        vm.prank(alice);
        registry.register("s1");
        vm.prank(alice);
        registry.update(50, bob, "v1");
        vm.prank(bob);
        registry.update(50, alice, "v2");
        assertEq(registry.getEntry(50).owner, alice);
    }

    function test_Update_RevertsWhenNotOwnerExistingEntry() public {
        vm.prank(alice);
        registry.register("s1");
        vm.prank(alice);
        registry.update(50, bob, "v1");
        vm.prank(alice);
        vm.expectRevert("not owner");
        registry.update(50, bob, "v2");
    }

    function test_Update_RevertsWhenOwnerIsZero() public {
        vm.prank(alice);
        registry.register("s1");
        vm.prank(alice);
        vm.expectRevert("invalid owner");
        registry.update(50, address(0), "");
    }

    function test_Register_CanPushMultiple() public {
        vm.startPrank(alice);
        for (uint256 i = 0; i < 5; i++) {
            registry.register("s");
        }
        vm.stopPrank();
        assertEq(registry.next_ordinal(), 1280);
    }

    function test_Register_ReturnedRangeMatchesStorage() public {
        vm.prank(alice);
        MNSRegistry.Range memory r = registry.register("s1");
        MNSRegistry.Range memory stored = registry.getRange(0);
        assertEq(r.ordinal, stored.ordinal);
        assertEq(r.owner, stored.owner);
        assertEq(r.nameServer, stored.nameServer);
    }

    function test_Register_RevertsWhenNameServerTooLong() public {
        vm.prank(alice);
        vm.expectRevert("name server too long");
        registry.register(newString(256));
    }

    function test_Update_RevertsWhenNameServerTooLong() public {
        vm.prank(alice);
        registry.register("s1");
        vm.prank(alice);
        vm.expectRevert("name server too long");
        registry.update(0, bob, newString(256));
    }

    function test_UpdateRange_UpdatesOwnerAndNameServer() public {
        vm.prank(alice);
        registry.register("s1");
        vm.prank(alice);
        registry.updateRange(0, bob, "ns1");
        MNSRegistry.Range memory r = registry.getRange(0);
        assertEq(r.owner, bob);
        assertEq(r.nameServer, "ns1");
    }

    function test_UpdateRange_RangeDefaultUpdatesGetNameServer() public {
        vm.prank(alice);
        registry.register("s1");
        vm.prank(alice);
        registry.updateRange(0, alice, "ns1");
        assertEq(registry.getNameServer(50), "ns1");
    }

    function test_UpdateRange_DoesNotAffectExistingEntry() public {
        vm.prank(alice);
        registry.register("s1");
        vm.prank(alice);
        registry.update(50, alice, "override");
        vm.prank(alice);
        registry.updateRange(0, alice, "newDefault");
        assertEq(registry.getNameServer(50), "override");
        assertEq(registry.getNameServer(51), "newDefault");
    }

    function test_UpdateRange_RevertsWhenIndexOutOfBounds() public {
        vm.prank(alice);
        vm.expectRevert("invalid range");
        registry.updateRange(0, alice, "ns1");
    }

    function test_UpdateRange_RevertsWhenNotOwner() public {
        vm.prank(alice);
        registry.register("s1");
        vm.prank(bob);
        vm.expectRevert("not owner");
        registry.updateRange(0, bob, "ns1");
    }

    function test_UpdateRange_RevertsWhenOwnerIsZero() public {
        vm.prank(alice);
        registry.register("s1");
        vm.prank(alice);
        vm.expectRevert("invalid owner");
        registry.updateRange(0, address(0), "ns1");
    }

    function test_UpdateRange_RevertsWhenNameServerTooLong() public {
        vm.prank(alice);
        registry.register("s1");
        vm.prank(alice);
        vm.expectRevert("name server too long");
        registry.updateRange(0, alice, newString(256));
    }

    function test_UpdateRange_NewOwnerCanUpdateEntries() public {
        vm.prank(alice);
        registry.register("s1");
        vm.prank(alice);
        registry.updateRange(0, bob, "ns1");
        vm.prank(bob);
        registry.update(50, bob, "entry");
        assertEq(registry.getNameServer(50), "entry");
    }

    function test_CanRegister_ReturnsTrueWhenTokensAvailable() public {
        assertTrue(registry.canRegister());
    }

    function test_CanRegister_ReturnsFalseWhenBucketEmpty() public {
        for (uint256 i = 0; i < 512; i++) {
            registry.register("s");
        }
        assertFalse(registry.canRegister());
    }

    function test_CanRegister_ReturnsTrueAfterRefill() public {
        for (uint256 i = 0; i < 512; i++) {
            registry.register("s");
        }
        assertFalse(registry.canRegister());
        vm.warp(block.timestamp + 1);
        assertTrue(registry.canRegister());
    }

    function test_Register_RevertsWhenBucketEmpty() public {
        for (uint256 i = 0; i < 512; i++) {
            registry.register("s");
        }
        vm.expectRevert("rate limit: try later");
        registry.register("s");
    }

    function test_Register_RefillsOneTokenAfterElapsed() public {
        for (uint256 i = 0; i < 512; i++) {
            registry.register("s");
        }
        // Refill time for 1 token = 86400 / 2^20 ≈ 0.082 seconds
        vm.warp(block.timestamp + 1);
        registry.register("s");
    }

    function test_Register_BucketCapsAtCapacity() public {
        for (uint256 i = 0; i < 512; i++) {
            registry.register("s");
        }
        // Warp a full day — bucket should refill to BUCKET_CAPACITY (512), no more.
        vm.warp(block.timestamp + 1 days);
        for (uint256 i = 0; i < 512; i++) {
            registry.register("s");
        }
        vm.expectRevert("rate limit: try later");
        registry.register("s");
    }

    function newString(uint256 len) internal pure returns (string memory) {
        bytes memory s = new bytes(len);
        for (uint256 i = 0; i < len; i++) {
            s[i] = "a";
        }
        return string(s);
    }
}
