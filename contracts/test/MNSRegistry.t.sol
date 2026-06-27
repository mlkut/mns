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

    function test_GetNS_ReturnsBatchNS() public {
        vm.prank(alice);
        registry.register(bytes32(0), "s1");
        MNSRegistry.ZoneConfig memory ns = registry.getZoneConfig(0);
        assertEq(ns.ns, "s1");
    }

    function test_GetNS_RoundsDownToCorrectBatch() public {
        vm.prank(alice);
        registry.register(bytes32(0), "s1");
        vm.prank(bob);
        registry.register(bytes32(0), "s2");
        MNSRegistry.ZoneConfig memory ns100 = registry.getZoneConfig(100);
        MNSRegistry.ZoneConfig memory ns300 = registry.getZoneConfig(300);
        assertEq(ns100.ns, "s1");
        assertEq(ns300.ns, "s2");
    }

    function test_GetNS_RevertsWhenOrdinalOutOfBatch() public {
        vm.prank(alice);
        registry.register(bytes32(0), "s1");
        vm.prank(bob);
        registry.register(bytes32(0), "s2");
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
        MNSRegistry.ZoneConfig memory ns = registry.getZoneConfig(50);
        assertEq(ns.ns, "overridden");
    }

    function test_GetNS_ReturnsBatchNSWhenNoEntry() public {
        vm.prank(alice);
        registry.register(bytes32(0), "s1");
        vm.prank(alice);
        registry.update(50, bob, bytes32(0), "overridden");
        MNSRegistry.ZoneConfig memory ns = registry.getZoneConfig(51);
        assertEq(ns.ns, "s1");
    }

    function test_GetNS_BoundaryExactMatch() public {
        vm.prank(alice);
        registry.register(bytes32(0), "s1");
        vm.prank(bob);
        registry.register(bytes32(0), "s2");
        MNSRegistry.ZoneConfig memory ns = registry.getZoneConfig(256);
        assertEq(ns.ns, "s2");
    }

    function test_Entry_SetsEntryNSForOrdinal() public {
        vm.prank(alice);
        registry.register(bytes32(0), "s1");
        vm.prank(alice);
        registry.update(50, bob, bytes32(0), "custom");
        assertEq(registry.getOwner(50), bob);
        MNSRegistry.ZoneConfig memory ns = registry.getZoneConfig(50);
        assertEq(ns.ns, "custom");
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
        MNSRegistry.ZoneConfig memory ns = registry.getZoneConfig(50);
        assertEq(ns.ns, "v2");
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
        MNSRegistry.ZoneConfig memory ns = registry.getZoneConfig(0);
        assertEq(ns.ns, "s1");
    }

    function test_Register_RevertsWhenNSTooLong() public {
        vm.prank(alice);
        vm.expectRevert("ns too long");
        registry.register(bytes32(0), newString(256));
    }

    function test_UpdateBatch_UpdatesOwnerAndNS() public {
        vm.prank(alice);
        registry.register(bytes32(0), "s1");
        vm.prank(alice);
        registry.updateBatch(0, bob, bytes32(0), "ns1");
        assertEq(registry.getOwner(0), bob);
        MNSRegistry.ZoneConfig memory ns = registry.getZoneConfig(0);
        assertEq(ns.ns, "ns1");
    }

    function test_UpdateBatch_BatchDefaultUpdatesGetNS() public {
        vm.prank(alice);
        registry.register(bytes32(0), "s1");
        vm.prank(alice);
        registry.updateBatch(0, alice, bytes32(0), "ns1");
        MNSRegistry.ZoneConfig memory ns = registry.getZoneConfig(50);
        assertEq(ns.ns, "ns1");
    }

    function test_UpdateBatch_DoesNotAffectExistingEntry() public {
        vm.prank(alice);
        registry.register(bytes32(0), "s1");
        vm.prank(alice);
        registry.update(50, alice, bytes32(0), "override");
        vm.prank(alice);
        registry.updateBatch(0, alice, bytes32(0), "newDefault");
        MNSRegistry.ZoneConfig memory ns50 = registry.getZoneConfig(50);
        MNSRegistry.ZoneConfig memory ns51 = registry.getZoneConfig(51);
        assertEq(ns50.ns, "override");
        assertEq(ns51.ns, "newDefault");
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
        MNSRegistry.ZoneConfig memory ns = registry.getZoneConfig(50);
        assertEq(ns.ns, "entry");
    }

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

    function test_Entry_SetsAndUpdatesSignerHash() public {
        vm.prank(alice);
        registry.register(bytes32(0), "s1");
        bytes32 hash1 = keccak256("key1");
        vm.prank(alice);
        registry.update(50, alice, hash1, "ns1");
        MNSRegistry.ZoneConfig memory c1 = registry.getZoneConfig(50);
        assertEq(c1.zsk, hash1);
        bytes32 hash2 = keccak256("key2");
        vm.prank(alice);
        registry.update(50, alice, hash2, "ns2");
        MNSRegistry.ZoneConfig memory c2 = registry.getZoneConfig(50);
        assertEq(c2.zsk, hash2);
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

    function newString(uint256 len) internal pure returns (string memory) {
        bytes memory s = new bytes(len);
        for (uint256 i = 0; i < len; i++) {
            s[i] = "a";
        }
        return string(s);
    }

    function test_Multicall_BatchOwnerCreatesMultipleEntries() public {
        vm.prank(alice);
        registry.register(bytes32(0), "s1");

        bytes[] memory calls = new bytes[](3);
        calls[0] = abi.encodeCall(registry.update, (10, alice, bytes32(0), "e10"));
        calls[1] = abi.encodeCall(registry.update, (20, alice, bytes32(0), "e20"));
        calls[2] = abi.encodeCall(registry.update, (30, alice, bytes32(0), "e30"));

        vm.prank(alice);
        registry.multicall(calls);

        assertEq(registry.getZoneConfig(10).ns, "e10");
        assertEq(registry.getZoneConfig(20).ns, "e20");
        assertEq(registry.getZoneConfig(30).ns, "e30");
    }

    function test_Multicall_PreservesMsgSender() public {
        vm.prank(alice);
        registry.register(bytes32(0), "s1");

        // bob is NOT the batch owner, so any update on this batch should revert
        bytes[] memory calls = new bytes[](1);
        calls[0] = abi.encodeCall(registry.update, (10, bob, bytes32(0), "hijack"));

        vm.prank(bob);
        vm.expectRevert("not owner");
        registry.multicall(calls);
    }

    function test_Multicall_RevertsAndRollsBackOnFailure() public {
        vm.prank(alice);
        registry.register(bytes32(0), "s1"); // batch 0, ordinals 0-255, alice owns it
        vm.prank(bob);
        registry.register(bytes32(0), "s2"); // batch 1, ordinals 256-511, bob owns it

        bytes[] memory calls = new bytes[](3);
        calls[0] = abi.encodeCall(registry.update, (10, alice, bytes32(0), "e10"));
        calls[1] = abi.encodeCall(registry.update, (20, alice, bytes32(0), "e20"));
        // alice tries to write into bob's batch — should revert with "not owner"
        calls[2] = abi.encodeCall(registry.update, (300, alice, bytes32(0), "bad"));

        vm.prank(alice);
        vm.expectRevert("not owner");
        registry.multicall(calls);

        // rollback check: entries were not persisted, ns falls back to batch defaults
        assertEq(registry.getZoneConfig(10).ns, "s1");
        assertEq(registry.getZoneConfig(20).ns, "s1");
        // owner falls back to batch owner (alice), no entry was created
        assertEq(registry.getOwner(10), alice);
        assertEq(registry.getOwner(20), alice);
    }

    function test_Multicall_RegisterAndUpdateInOneTx() public {
        // Demonstrates the registrar pattern: register a batch and immediately
        // carve out several entries for users, all in one transaction.
        bytes[] memory calls = new bytes[](4);
        calls[0] = abi.encodeCall(registry.register, (bytes32(0), "registrar.ns"));
        calls[1] = abi.encodeCall(registry.update, (0, bob, bytes32(0), "user0"));
        calls[2] = abi.encodeCall(registry.update, (1, bob, bytes32(0), "user1"));
        calls[3] = abi.encodeCall(registry.update, (100, alice, bytes32(0), "user100"));

        vm.prank(alice);
        registry.multicall(calls);

        assertEq(registry.getOwner(0), bob);
        assertEq(registry.getOwner(1), bob);
        assertEq(registry.getOwner(100), alice);
        assertEq(registry.getZoneConfig(50).ns, "registrar.ns"); // batch default intact
    }
}
