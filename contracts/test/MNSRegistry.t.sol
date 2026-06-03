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
        assertEq(r.ns.nameServer, "s1");
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
        string memory nameServer = registry.getNameserverConfig(0).nameServer;
        assertEq(nameServer, "s1");
    }

    function test_GetNameServer_RoundsDownToCorrectRange() public {
        vm.prank(alice);
        registry.register("s1");
        vm.prank(bob);
        registry.register("s2");
        assertEq(registry.getNameserverConfig(100).nameServer, "s1");
        assertEq(registry.getNameserverConfig(300).nameServer, "s2");
    }

    function test_GetNameServer_RevertsWhenOrdinalOutOfRange() public {
        vm.prank(alice);
        registry.register("s1");
        vm.prank(bob);
        registry.register("s2");
        vm.expectRevert("ordinal out of range");
        registry.getNameserverConfig(100000);
    }

    function test_GetNameServer_ReturnsEntryNameServer() public {
        vm.prank(alice);
        registry.register("s1");
        vm.prank(alice);
        registry.update(50, bob, "overridden", bytes32(0));
        assertEq(registry.getNameserverConfig(50).nameServer, "overridden");
    }

    function test_GetNameServer_ReturnsRangeNameServerWhenNoEntry() public {
        vm.prank(alice);
        registry.register("s1");
        vm.prank(alice);
        registry.update(50, bob, "overridden", bytes32(0));
        assertEq(registry.getNameserverConfig(51).nameServer, "s1");
    }

    function test_GetNameServer_BoundaryExactMatch() public {
        vm.prank(alice);
        registry.register("s1");
        vm.prank(bob);
        registry.register("s2");
        assertEq(registry.getNameserverConfig(256).nameServer, "s2");
    }

    function test_Entry_SetsEntryNameServerForOrdinal() public {
        vm.prank(alice);
        registry.register("s1");
        vm.prank(alice);
        registry.update(50, bob, "custom", bytes32(0));
        MNSRegistry.Entry memory e = registry.getEntry(50);
        assertEq(e.owner, bob);
        assertEq(e.ns.nameServer, "custom");
    }

    function test_Update_RangeOwnerCreatesEntry() public {
        vm.prank(alice);
        registry.register("s1");
        vm.prank(alice);
        registry.update(50, bob, "ok", bytes32(0));
        assertEq(registry.getEntry(50).owner, bob);
    }

    function test_Update_RevertsWhenNotOwnerNoEntry() public {
        vm.prank(alice);
        registry.register("s1");
        vm.prank(bob);
        vm.expectRevert("not owner");
        registry.update(50, bob, "valid", bytes32(0));
    }

    function test_Update_EntryOwnerCanUpdate() public {
        vm.prank(alice);
        registry.register("s1");
        vm.prank(alice);
        registry.update(50, bob, "v1", bytes32(0));
        vm.prank(bob);
        registry.update(50, alice, "v2", bytes32(0));
        assertEq(registry.getEntry(50).owner, alice);
    }

    function test_Update_RevertsWhenNotOwnerExistingEntry() public {
        vm.prank(alice);
        registry.register("s1");
        vm.prank(alice);
        registry.update(50, bob, "v1", bytes32(0));
        vm.prank(alice);
        vm.expectRevert("not owner");
        registry.update(50, bob, "v2", bytes32(0));
    }

    function test_Update_RevertsWhenOwnerIsZero() public {
        vm.prank(alice);
        registry.register("s1");
        vm.prank(alice);
        vm.expectRevert("invalid owner");
        registry.update(50, address(0), "", bytes32(0));
    }

    function test_Register_CanPushMultiple() public {
        vm.startPrank(alice);
        for (uint256 i = 0; i < 5; i++) {
            registry.register("s");
        }
        vm.stopPrank();
        assertEq(registry.nextOrdinal(), 1280);
    }

    function test_Register_ReturnedRangeMatchesStorage() public {
        vm.prank(alice);
        MNSRegistry.Range memory r = registry.register("s1");
        MNSRegistry.Range memory stored = registry.getRange(0);
        assertEq(r.ordinal, stored.ordinal);
        assertEq(r.owner, stored.owner);
        assertEq(r.ns.nameServer, stored.ns.nameServer);
    }

    function test_Register_RevertsWhenNameServerTooLong() public {
        vm.prank(alice);
        vm.expectRevert("nameserver too long");
        registry.register(newString(256));
    }

    function test_Update_RevertsWhenNameServerTooLong() public {
        vm.prank(alice);
        registry.register("s1");
        vm.prank(alice);
        vm.expectRevert("nameserver too long");
        registry.update(0, bob, newString(256), bytes32(0));
    }

    function test_UpdateRange_UpdatesOwnerAndNameServer() public {
        vm.prank(alice);
        registry.register("s1");
        vm.prank(alice);
        registry.updateRange(0, bob, "ns1");
        MNSRegistry.Range memory r = registry.getRange(0);
        assertEq(r.owner, bob);
        assertEq(r.ns.nameServer, "ns1");
    }

    function test_UpdateRange_RangeDefaultUpdatesGetNameServer() public {
        vm.prank(alice);
        registry.register("s1");
        vm.prank(alice);
        registry.updateRange(0, alice, "ns1");
        assertEq(registry.getNameserverConfig(50).nameServer, "ns1");
    }

    function test_UpdateRange_DoesNotAffectExistingEntry() public {
        vm.prank(alice);
        registry.register("s1");
        vm.prank(alice);
        registry.update(50, alice, "override", bytes32(0));
        vm.prank(alice);
        registry.updateRange(0, alice, "newDefault");
        assertEq(registry.getNameserverConfig(50).nameServer, "override");
        assertEq(registry.getNameserverConfig(51).nameServer, "newDefault");
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
        vm.expectRevert("nameserver too long");
        registry.updateRange(0, alice, newString(256));
    }

    function test_UpdateRange_NewOwnerCanUpdateEntries() public {
        vm.prank(alice);
        registry.register("s1");
        vm.prank(alice);
        registry.updateRange(0, bob, "ns1");
        vm.prank(bob);
        registry.update(50, bob, "entry", bytes32(0));
        assertEq(registry.getNameserverConfig(50).nameServer, "entry");
    }

    function test_CanRegister_ReturnsTrueWhenTokensAvailable() public {
        assertTrue(registry.canRegister());
    }

    function _exhaustBucket() internal {
        for (uint256 i = 0; i < 512; i++) {
            registry.register("s");
            if (i % 10 == 9) vm.roll(block.number + 1);
        }
    }

    function test_CanRegister_ReturnsFalseWhenBucketEmpty() public {
        _exhaustBucket();
        assertFalse(registry.canRegister());
    }

    function test_CanRegister_ReturnsTrueAfterRefill() public {
        _exhaustBucket();
        assertFalse(registry.canRegister());
        vm.warp(block.timestamp + 1);
        assertTrue(registry.canRegister());
    }

    function test_Register_RevertsWhenBucketEmpty() public {
        _exhaustBucket();
        vm.expectRevert("rate limit: daily cap");
        registry.register("s");
    }

    function test_Register_RefillsOneTokenAfterElapsed() public {
        _exhaustBucket();
        // Refill time for 1 token = 86400 / 2^20 ≈ 0.082 seconds
        vm.warp(block.timestamp + 1);
        registry.register("s");
    }

    function test_Register_BucketCapsAtCapacity() public {
        _exhaustBucket();
        // Warp a full day — bucket should refill to BUCKET_CAPACITY (512), no more.
        vm.warp(block.timestamp + 1 days);
        vm.roll(block.number + 1);
        _exhaustBucket();
        vm.expectRevert("rate limit: daily cap");
        registry.register("s");
    }

    function test_Entry_SetsAndUpdatesSignerHash() public {
        vm.prank(alice);
        registry.register("s1");
        bytes32 hash1 = keccak256("key1");
        vm.prank(alice);
        registry.update(50, alice, "ns1", hash1);
        MNSRegistry.Entry memory e = registry.getEntry(50);
        assertEq(e.ns.signerHash, hash1);
        bytes32 hash2 = keccak256("key2");
        vm.prank(alice);
        registry.update(50, alice, "ns2", hash2);
        e = registry.getEntry(50);
        assertEq(e.ns.signerHash, hash2);
    }

    function test_GetNameserverConfig_ReturnsFullConfigForEntry() public {
        vm.prank(alice);
        registry.register("s1");
        bytes32 hashVal = keccak256("signer");
        vm.prank(alice);
        registry.update(50, alice, "custom", hashVal);
        MNSRegistry.NameserverConfig memory cfg = registry.getNameserverConfig(50);
        assertEq(cfg.nameServer, "custom");
        assertEq(cfg.signerHash, hashVal);
    }

    function test_GetNameserverConfig_SignerHashIsZeroForRange() public {
        vm.prank(alice);
        registry.register("s1");
        MNSRegistry.NameserverConfig memory cfg = registry.getNameserverConfig(0);
        assertEq(cfg.nameServer, "s1");
        assertEq(cfg.signerHash, bytes32(0));
    }

    function test_Register_RevertsWhenNameServerEmpty() public {
        vm.prank(alice);
        vm.expectRevert("empty nameserver");
        registry.register("");
    }

    function test_Update_RevertsWhenNameServerEmpty() public {
        vm.prank(alice);
        registry.register("s1");
        vm.prank(alice);
        vm.expectRevert("empty nameserver");
        registry.update(0, alice, "", bytes32(0));
    }

    function newString(uint256 len) internal pure returns (string memory) {
        bytes memory s = new bytes(len);
        for (uint256 i = 0; i < len; i++) {
            s[i] = "a";
        }
        return string(s);
    }
}
