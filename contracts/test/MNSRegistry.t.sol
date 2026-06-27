// SPDX-License-Identifier: MIT
pragma solidity ^0.8.33;

import {Test} from "forge-std/Test.sol";
import {MNSRegistry} from "../src/MNSRegistry.sol";

contract MNSRegistryTest is Test {
    MNSRegistry registry;
    address alice = address(0xA11CE);
    address bob = address(0xB0B);

    function setUp() public {
        registry = new MNSRegistry();
    }

    // ─────────────────────────────────────────────────────────────────────────
    // register
    // ─────────────────────────────────────────────────────────────────────────

    function test_Register_ReturnsBatchStartingAtZero() public {
        vm.prank(alice);
        MNSRegistry.Batch memory r = registry.register(bytes32(0), "s1");
        assertEq(r.ordinal, 0);
        assertEq(r.owner, alice);
        assertEq(r.zone.ns, "s1");
    }

    function test_Register_SubsequentIncrementsBy256() public {
        vm.prank(alice);
        MNSRegistry.Batch memory r1 = registry.register(bytes32(0), "s1");
        vm.prank(bob);
        MNSRegistry.Batch memory r2 = registry.register(bytes32(0), "s2");
        assertEq(r1.ordinal, 0);
        assertEq(r2.ordinal, 256);
    }

    function test_Register_PushesToBatchesArray() public {
        vm.prank(alice);
        registry.register(bytes32(0), "s1");
        assertEq(registry.nextOrdinal(), 256);
    }

    function test_Register_CanPushMultiple() public {
        vm.startPrank(alice);
        registry.register(bytes32(0), "s");
        registry.register(bytes32(0), "s");
        vm.stopPrank();
        assertEq(registry.nextOrdinal(), 512);
    }

    function test_Register_ReturnedBatchMatchesStorage() public {
        vm.prank(alice);
        MNSRegistry.Batch memory r = registry.register(bytes32(0), "s1");
        assertEq(r.ordinal, 0);
        assertEq(r.owner, registry.getOwner(0));
        assertEq(registry.getZoneConfig(0).ns, "s1");
    }

    function test_Register_RevertsWhenNSTooLong() public {
        vm.prank(alice);
        vm.expectRevert("ns too long");
        registry.register(bytes32(0), newString(256));
    }

    // ─────────────────────────────────────────────────────────────────────────
    // getZoneConfig / getOwner
    // ─────────────────────────────────────────────────────────────────────────

    function test_GetNS_ReturnsBatchNS() public {
        vm.prank(alice);
        registry.register(bytes32(0), "s1");
        assertEq(registry.getZoneConfig(0).ns, "s1");
    }

    function test_GetNS_RoundsDownToCorrectBatch() public {
        vm.prank(alice);
        registry.register(bytes32(0), "s1");
        vm.prank(bob);
        registry.register(bytes32(0), "s2");
        assertEq(registry.getZoneConfig(100).ns, "s1");
        assertEq(registry.getZoneConfig(300).ns, "s2");
    }

    function test_GetNS_RevertsWhenOrdinalOutOfBatch() public {
        vm.prank(alice);
        registry.register(bytes32(0), "s1");
        vm.expectRevert(abi.encodeWithSelector(MNSRegistry.OrdinalNotRegistered.selector, 100000));
        registry.getZoneConfig(100000);
        vm.expectRevert(abi.encodeWithSelector(MNSRegistry.OrdinalNotRegistered.selector, 100000));
        registry.getOwner(100000);
    }

    function test_GetNS_ReturnsEntryNS() public {
        vm.prank(alice);
        registry.register(bytes32(0), "s1");
        vm.prank(alice);
        registry.update(50, bob, bytes32(0), "overridden");
        assertEq(registry.getZoneConfig(50).ns, "overridden");
    }

    function test_GetNS_ReturnsBatchNSWhenNoEntry() public {
        vm.prank(alice);
        registry.register(bytes32(0), "s1");
        vm.prank(alice);
        registry.update(50, bob, bytes32(0), "overridden");
        assertEq(registry.getZoneConfig(51).ns, "s1");
    }

    function test_GetNS_BoundaryExactMatch() public {
        vm.prank(alice);
        registry.register(bytes32(0), "s1");
        vm.prank(bob);
        registry.register(bytes32(0), "s2");
        assertEq(registry.getZoneConfig(256).ns, "s2");
    }

    function test_GetZoneConfig_ReturnsSignerHashAndNS() public {
        vm.prank(alice);
        registry.register(bytes32(0), "s1");
        bytes32 hashVal = keccak256("signer");
        vm.prank(alice);
        registry.update(50, alice, hashVal, "custom");
        MNSRegistry.ZoneConfig memory cfg = registry.getZoneConfig(50);
        assertEq(cfg.ns, "custom");
        assertEq(cfg.zsk, hashVal);
    }

    function test_GetZoneConfig_SignerHashIsZeroForBatch() public {
        vm.prank(alice);
        registry.register(bytes32(0), "s1");
        MNSRegistry.ZoneConfig memory cfg = registry.getZoneConfig(0);
        assertEq(cfg.ns, "s1");
        assertEq(cfg.zsk, bytes32(0));
    }

    // ─────────────────────────────────────────────────────────────────────────
    // getBatch / hasEntry
    // ─────────────────────────────────────────────────────────────────────────

    function test_GetBatch_ReturnsBatchForOrdinal() public {
        vm.prank(alice);
        registry.register(bytes32(0), "s1");
        MNSRegistry.Batch memory b = registry.getBatch(100);
        assertEq(b.ordinal, 0);
        assertEq(b.owner, alice);
        assertEq(b.zone.ns, "s1");
    }

    function test_GetBatch_RevertsWhenNotRegistered() public {
        vm.expectRevert(abi.encodeWithSelector(MNSRegistry.BatchNotRegistered.selector, 999));
        registry.getBatch(999);
    }

    function test_HasEntry_ReturnsFalseWithNoEntry() public {
        vm.prank(alice);
        registry.register(bytes32(0), "s1");
        assertFalse(registry.hasEntry(50));
    }

    function test_HasEntry_ReturnsTrueAfterUpdate() public {
        vm.prank(alice);
        registry.register(bytes32(0), "s1");
        vm.prank(alice);
        registry.update(50, alice, bytes32(0), "e50");
        assertTrue(registry.hasEntry(50));
        assertFalse(registry.hasEntry(51));
    }

    // ─────────────────────────────────────────────────────────────────────────
    // update
    // ─────────────────────────────────────────────────────────────────────────

    function test_Entry_SetsEntryNSForOrdinal() public {
        vm.prank(alice);
        registry.register(bytes32(0), "s1");
        vm.prank(alice);
        registry.update(50, bob, bytes32(0), "custom");
        assertEq(registry.getOwner(50), bob);
        assertEq(registry.getZoneConfig(50).ns, "custom");
    }

    function test_Update_BatchOwnerCreatesEntry() public {
        vm.prank(alice);
        registry.register(bytes32(0), "s1");
        vm.prank(alice);
        registry.update(50, bob, bytes32(0), "ok");
        assertEq(registry.getOwner(50), bob);
    }

    function test_Update_RevertsWhenNotOwnerNoEntry() public {
        vm.prank(alice);
        registry.register(bytes32(0), "s1");
        vm.prank(bob);
        vm.expectRevert("not owner");
        registry.update(50, bob, bytes32(0), "s1");
    }

    function test_Update_EntryOwnerCanUpdate() public {
        vm.prank(alice);
        registry.register(bytes32(0), "s1");
        vm.prank(alice);
        registry.update(50, bob, bytes32(0), "v1");
        vm.prank(bob);
        registry.update(50, alice, bytes32(0), "v2");
        assertEq(registry.getZoneConfig(50).ns, "v2");
        assertEq(registry.getOwner(50), alice);
    }

    function test_Update_RevertsWhenNotOwnerExistingEntry() public {
        vm.prank(alice);
        registry.register(bytes32(0), "s1");
        vm.prank(alice);
        registry.update(50, bob, bytes32(0), "v1");
        vm.prank(alice);
        vm.expectRevert("not owner");
        registry.update(50, bob, bytes32(0), "v2");
    }

    function test_Update_RevertsWhenOwnerIsZero() public {
        vm.prank(alice);
        registry.register(bytes32(0), "s1");
        vm.prank(alice);
        vm.expectRevert("invalid owner");
        registry.update(50, address(0), bytes32(0), "s1");
    }

    function test_Entry_SetsAndUpdatesSignerHash() public {
        vm.prank(alice);
        registry.register(bytes32(0), "s1");
        bytes32 hash1 = keccak256("key1");
        vm.prank(alice);
        registry.update(50, alice, hash1, "ns1");
        assertEq(registry.getZoneConfig(50).zsk, hash1);
        bytes32 hash2 = keccak256("key2");
        vm.prank(alice);
        registry.update(50, alice, hash2, "ns2");
        assertEq(registry.getZoneConfig(50).zsk, hash2);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // updateBatch
    // ─────────────────────────────────────────────────────────────────────────

    function test_UpdateBatch_UpdatesOwnerAndNS() public {
        vm.prank(alice);
        registry.register(bytes32(0), "s1");
        vm.prank(alice);
        registry.updateBatch(0, bob, bytes32(0), "ns1");
        assertEq(registry.getOwner(0), bob);
        assertEq(registry.getZoneConfig(0).ns, "ns1");
    }

    function test_UpdateBatch_BatchDefaultUpdatesGetNS() public {
        vm.prank(alice);
        registry.register(bytes32(0), "s1");
        vm.prank(alice);
        registry.updateBatch(0, alice, bytes32(0), "ns1");
        assertEq(registry.getZoneConfig(50).ns, "ns1");
    }

    function test_UpdateBatch_DoesNotAffectExistingEntry() public {
        vm.prank(alice);
        registry.register(bytes32(0), "s1");
        vm.prank(alice);
        registry.update(50, alice, bytes32(0), "override");
        vm.prank(alice);
        registry.updateBatch(0, alice, bytes32(0), "newDefault");
        assertEq(registry.getZoneConfig(50).ns, "override");
        assertEq(registry.getZoneConfig(51).ns, "newDefault");
    }

    function test_UpdateBatch_RevertsWhenOrdinalOutOfBatch() public {
        vm.prank(alice);
        registry.register(bytes32(0), "s1");
        vm.prank(alice);
        vm.expectRevert(abi.encodeWithSelector(MNSRegistry.BatchNotRegistered.selector, 256));
        registry.updateBatch(256, alice, bytes32(0), "s1");
    }

    function test_UpdateBatch_RevertsWhenNotOwner() public {
        vm.prank(alice);
        registry.register(bytes32(0), "s1");
        vm.prank(bob);
        vm.expectRevert("not owner");
        registry.updateBatch(0, bob, bytes32(0), "s1");
    }

    function test_UpdateBatch_RevertsWhenOwnerIsZero() public {
        vm.prank(alice);
        registry.register(bytes32(0), "s1");
        vm.prank(alice);
        vm.expectRevert("invalid owner");
        registry.updateBatch(0, address(0), bytes32(0), "s1");
    }

    function test_UpdateBatch_RevertsWhenNSTooLong() public {
        vm.prank(alice);
        registry.register(bytes32(0), "s1");
        vm.prank(alice);
        vm.expectRevert("ns too long");
        registry.updateBatch(0, alice, bytes32(0), newString(256));
    }

    function test_UpdateBatch_NewOwnerCanUpdateEntries() public {
        vm.prank(alice);
        registry.register(bytes32(0), "s1");
        vm.prank(alice);
        registry.updateBatch(0, bob, bytes32(0), "ns1");
        vm.prank(bob);
        registry.update(50, bob, bytes32(0), "entry");
        assertEq(registry.getZoneConfig(50).ns, "entry");
    }

    // ─────────────────────────────────────────────────────────────────────────
    // updateMany
    // ─────────────────────────────────────────────────────────────────────────

    function test_UpdateMany_BatchOwnerCreatesMultipleEntries() public {
        vm.prank(alice);
        registry.register(bytes32(0), "s1");

        MNSRegistry.EntryUpdate[] memory updates = new MNSRegistry.EntryUpdate[](3);
        updates[0] = MNSRegistry.EntryUpdate(10, alice, bytes32(0), "e10");
        updates[1] = MNSRegistry.EntryUpdate(20, alice, bytes32(0), "e20");
        updates[2] = MNSRegistry.EntryUpdate(30, alice, bytes32(0), "e30");

        vm.prank(alice);
        registry.updateMany(updates);

        assertEq(registry.getZoneConfig(10).ns, "e10");
        assertEq(registry.getZoneConfig(20).ns, "e20");
        assertEq(registry.getZoneConfig(30).ns, "e30");
    }

    function test_UpdateMany_RevertsWhenNotOwner() public {
        vm.prank(alice);
        registry.register(bytes32(0), "s1");

        MNSRegistry.EntryUpdate[] memory updates = new MNSRegistry.EntryUpdate[](1);
        updates[0] = MNSRegistry.EntryUpdate(10, bob, bytes32(0), "hijack");

        vm.prank(bob);
        vm.expectRevert("not owner");
        registry.updateMany(updates);
    }

    function test_UpdateMany_RevertsAndRollsBackOnFailure() public {
        vm.prank(alice);
        registry.register(bytes32(0), "s1"); // batch 0, ordinals 0-255, alice owns it
        vm.prank(bob);
        registry.register(bytes32(0), "s2"); // batch 1, ordinals 256-511, bob owns it

        MNSRegistry.EntryUpdate[] memory updates = new MNSRegistry.EntryUpdate[](3);
        updates[0] = MNSRegistry.EntryUpdate(10, alice, bytes32(0), "e10");
        updates[1] = MNSRegistry.EntryUpdate(20, alice, bytes32(0), "e20");
        updates[2] = MNSRegistry.EntryUpdate(300, alice, bytes32(0), "bad"); // bob's batch

        vm.prank(alice);
        vm.expectRevert("not owner");
        registry.updateMany(updates);

        // rollback check: entries were not persisted, ns falls back to batch defaults
        assertEq(registry.getZoneConfig(10).ns, "s1");
        assertEq(registry.getZoneConfig(20).ns, "s1");
        assertEq(registry.getOwner(10), alice);
        assertEq(registry.getOwner(20), alice);
    }

    function test_UpdateMany_RegistrarHandsOutEntriesToUsers() public {
        // Demonstrates the registrar pattern: register a batch then distribute
        // entries to users via updateMany. Registration stays as a standalone
        // call — it cannot be batched with updateMany by design.
        vm.prank(alice);
        registry.register(bytes32(0), "registrar.ns");

        MNSRegistry.EntryUpdate[] memory updates = new MNSRegistry.EntryUpdate[](3);
        updates[0] = MNSRegistry.EntryUpdate(0, bob, bytes32(0), "user0");
        updates[1] = MNSRegistry.EntryUpdate(1, bob, bytes32(0), "user1");
        updates[2] = MNSRegistry.EntryUpdate(100, alice, bytes32(0), "user100");

        vm.prank(alice);
        registry.updateMany(updates);

        assertEq(registry.getOwner(0), bob);
        assertEq(registry.getOwner(1), bob);
        assertEq(registry.getOwner(100), alice);
        assertEq(registry.getZoneConfig(50).ns, "registrar.ns"); // batch default intact
    }

    function test_UpdateMany_EmptyArraySucceeds() public {
        MNSRegistry.EntryUpdate[] memory updates = new MNSRegistry.EntryUpdate[](0);
        registry.updateMany(updates); // should not revert
    }

    // ─────────────────────────────────────────────────────────────────────────
    // rate limiter
    // ─────────────────────────────────────────────────────────────────────────

    function test_EstimatedWaitTime_ReturnsZeroAtStart() public view {
        assertEq(registry.estimatedWaitTime(), 0);
    }

    function _exhaustBucket() internal {
        while (registry.estimatedWaitTime() == 0) {
            registry.register(bytes32(0), "s");
        }
    }

    function test_EstimatedWaitTime_ReturnsPositiveWhenBucketEmpty() public {
        _exhaustBucket();
        assertGt(registry.estimatedWaitTime(), 0);
    }

    function test_EstimatedWaitTime_ReturnsZeroAfterRefill() public {
        _exhaustBucket();
        assertGt(registry.estimatedWaitTime(), 0);
        vm.warp(block.timestamp + 22);
        assertEq(registry.estimatedWaitTime(), 0);
    }

    function test_Register_RevertsWhenBucketEmpty() public {
        _exhaustBucket();
        vm.expectRevert(abi.encodeWithSelector(MNSRegistry.RateLimit.selector, 22));
        registry.register(bytes32(0), "s");
    }

    function test_Register_RefillsForRegistrationAfterElapsed() public {
        _exhaustBucket();
        vm.warp(block.timestamp + 22);
        registry.register(bytes32(0), "s");
    }

    function test_Register_BucketCapsAtCapacity() public {
        _exhaustBucket();
        vm.warp(block.timestamp + 1 days);
        _exhaustBucket();
        vm.expectRevert(abi.encodeWithSelector(MNSRegistry.RateLimit.selector, 22));
        registry.register(bytes32(0), "s");
    }

    function test_Register_RefillsAfterBurstAcrossBlocks() public {
        assertEq(registry.estimatedWaitTime(), 0);
        _exhaustBucket();
        assertGt(registry.estimatedWaitTime(), 0);
        vm.warp(block.timestamp + 22);
        assertEq(registry.estimatedWaitTime(), 0);
        registry.register(bytes32(0), "s");
    }

    function test_ConsumeBucketToken_CarriesOverFractionalAccrual() public {
        _exhaustBucket();
        vm.warp(block.timestamp + 22);
        registry.register(bytes32(0), "s");
        vm.warp(block.timestamp + 20);
        assertEq(registry.estimatedWaitTime(), 0);
        registry.register(bytes32(0), "s");
    }

    // ─────────────────────────────────────────────────────────────────────────
    // helpers
    // ─────────────────────────────────────────────────────────────────────────

    function newString(uint256 len) internal pure returns (string memory) {
        bytes memory s = new bytes(len);
        for (uint256 i = 0; i < len; i++) {
            s[i] = "a";
        }
        return string(s);
    }
}
