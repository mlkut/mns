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
        MNSRegistry.Batch memory r = registry.register("s1", bytes32(0));
        assertEq(r.ordinal, 0);
        assertEq(r.owner, alice);
        assertEq(r.ns.nameServer, "s1");
    }

    function test_Register_SubsequentIncrementsBy256() public {
        vm.prank(alice);
        MNSRegistry.Batch memory r1 = registry.register("s1", bytes32(0));
        vm.prank(bob);
        MNSRegistry.Batch memory r2 = registry.register("s2", bytes32(0));
        assertEq(r1.ordinal, 0);
        assertEq(r2.ordinal, 256);
    }

    function test_Register_PushesToBatchesArray() public {
        vm.prank(alice);
        registry.register("s1", bytes32(0));
        assertEq(registry.getBatch(0).ordinal, 0);
    }

    function test_GetNameServer_ReturnsBatchNameServer() public {
        vm.prank(alice);
        registry.register("s1", bytes32(0));
        string memory nameServer = registry.getNameserverConfig(0).nameServer;
        assertEq(nameServer, "s1");
    }

    function test_GetNameServer_RoundsDownToCorrectBatch() public {
        vm.prank(alice);
        registry.register("s1", bytes32(0));
        vm.prank(bob);
        registry.register("s2", bytes32(0));
        assertEq(registry.getNameserverConfig(100).nameServer, "s1");
        assertEq(registry.getNameserverConfig(300).nameServer, "s2");
    }

    function test_GetNameServer_RevertsWhenOrdinalOutOfBatch() public {
        vm.prank(alice);
        registry.register("s1", bytes32(0));
        vm.prank(bob);
        registry.register("s2", bytes32(0));
        vm.expectRevert("ordinal out of batch");
        registry.getNameserverConfig(100000);
    }

    function test_GetNameServer_ReturnsEntryNameServer() public {
        vm.prank(alice);
        registry.register("s1", bytes32(0));
        vm.prank(alice);
        registry.update(50, bob, "overridden", bytes32(0));
        assertEq(registry.getNameserverConfig(50).nameServer, "overridden");
    }

    function test_GetNameServer_ReturnsBatchNameServerWhenNoEntry() public {
        vm.prank(alice);
        registry.register("s1", bytes32(0));
        vm.prank(alice);
        registry.update(50, bob, "overridden", bytes32(0));
        assertEq(registry.getNameserverConfig(51).nameServer, "s1");
    }

    function test_GetNameServer_BoundaryExactMatch() public {
        vm.prank(alice);
        registry.register("s1", bytes32(0));
        vm.prank(bob);
        registry.register("s2", bytes32(0));
        assertEq(registry.getNameserverConfig(256).nameServer, "s2");
    }

    function test_Entry_SetsEntryNameServerForOrdinal() public {
        vm.prank(alice);
        registry.register("s1", bytes32(0));
        vm.prank(alice);
        registry.update(50, bob, "custom", bytes32(0));
        MNSRegistry.Entry memory e = registry.getEntry(50);
        assertEq(e.owner, bob);
        assertEq(e.ns.nameServer, "custom");
    }

    function test_Update_BatchOwnerCreatesEntry() public {
        vm.prank(alice);
        registry.register("s1", bytes32(0));
        vm.prank(alice);
        registry.update(50, bob, "ok", bytes32(0));
        assertEq(registry.getEntry(50).owner, bob);
    }

    function test_Update_RevertsWhenNotOwnerNoEntry() public {
        vm.prank(alice);
        registry.register("s1", bytes32(0));
        vm.prank(bob);
        vm.expectRevert("not owner");
        registry.update(50, bob, "valid", bytes32(0));
    }

    function test_Update_EntryOwnerCanUpdate() public {
        vm.prank(alice);
        registry.register("s1", bytes32(0));
        vm.prank(alice);
        registry.update(50, bob, "v1", bytes32(0));
        vm.prank(bob);
        registry.update(50, alice, "v2", bytes32(0));
        assertEq(registry.getEntry(50).owner, alice);
    }

    function test_Update_RevertsWhenNotOwnerExistingEntry() public {
        vm.prank(alice);
        registry.register("s1", bytes32(0));
        vm.prank(alice);
        registry.update(50, bob, "v1", bytes32(0));
        vm.prank(alice);
        vm.expectRevert("not owner");
        registry.update(50, bob, "v2", bytes32(0));
    }

    function test_Update_RevertsWhenOwnerIsZero() public {
        vm.prank(alice);
        registry.register("s1", bytes32(0));
        vm.prank(alice);
        vm.expectRevert("invalid owner");
        registry.update(50, address(0), "", bytes32(0));
    }

    function test_Register_CanPushMultiple() public {
        vm.startPrank(alice);
        registry.register("s", bytes32(0));
        registry.register("s", bytes32(0));
        vm.stopPrank();
        assertEq(registry.nextOrdinal(), 512);
    }

    function test_Register_ReturnedBatchMatchesStorage() public {
        vm.prank(alice);
        MNSRegistry.Batch memory r = registry.register("s1", bytes32(0));
        MNSRegistry.Batch memory stored = registry.getBatch(0);
        assertEq(r.ordinal, stored.ordinal);
        assertEq(r.owner, stored.owner);
        assertEq(r.ns.nameServer, stored.ns.nameServer);
    }

    function test_Register_RevertsWhenNameServerTooLong() public {
        vm.prank(alice);
        vm.expectRevert("nameserver too long");
        registry.register(newString(256), bytes32(0));
    }

    function test_Update_RevertsWhenNameServerTooLong() public {
        vm.prank(alice);
        registry.register("s1", bytes32(0));
        vm.prank(alice);
        vm.expectRevert("nameserver too long");
        registry.update(0, bob, newString(256), bytes32(0));
    }

    function test_UpdateBatch_UpdatesOwnerAndNameServer() public {
        vm.prank(alice);
        registry.register("s1", bytes32(0));
        vm.prank(alice);
        registry.updateBatch(0, bob, "ns1", bytes32(0));
        MNSRegistry.Batch memory r = registry.getBatch(0);
        assertEq(r.owner, bob);
        assertEq(r.ns.nameServer, "ns1");
    }

    function test_UpdateBatch_BatchDefaultUpdatesGetNameServer() public {
        vm.prank(alice);
        registry.register("s1", bytes32(0));
        vm.prank(alice);
        registry.updateBatch(0, alice, "ns1", bytes32(0));
        assertEq(registry.getNameserverConfig(50).nameServer, "ns1");
    }

    function test_UpdateBatch_DoesNotAffectExistingEntry() public {
        vm.prank(alice);
        registry.register("s1", bytes32(0));
        vm.prank(alice);
        registry.update(50, alice, "override", bytes32(0));
        vm.prank(alice);
        registry.updateBatch(0, alice, "newDefault", bytes32(0));
        assertEq(registry.getNameserverConfig(50).nameServer, "override");
        assertEq(registry.getNameserverConfig(51).nameServer, "newDefault");
    }

    function test_UpdateBatch_RevertsWhenOrdinalOutOfBatch() public {
        vm.prank(alice);
        registry.register("s1", bytes32(0));
        vm.prank(alice);
        vm.expectRevert("ordinal out of batch");
        registry.updateBatch(256, alice, "ns1", bytes32(0));
    }

    function test_UpdateBatch_RevertsWhenNotOwner() public {
        vm.prank(alice);
        registry.register("s1", bytes32(0));
        vm.prank(bob);
        vm.expectRevert("not owner");
        registry.updateBatch(0, bob, "ns1", bytes32(0));
    }

    function test_UpdateBatch_RevertsWhenOwnerIsZero() public {
        vm.prank(alice);
        registry.register("s1", bytes32(0));
        vm.prank(alice);
        vm.expectRevert("invalid owner");
        registry.updateBatch(0, address(0), "ns1", bytes32(0));
    }

    function test_UpdateBatch_RevertsWhenNameServerTooLong() public {
        vm.prank(alice);
        registry.register("s1", bytes32(0));
        vm.prank(alice);
        vm.expectRevert("nameserver too long");
        registry.updateBatch(0, alice, newString(256), bytes32(0));
    }

    function test_UpdateBatch_NewOwnerCanUpdateEntries() public {
        vm.prank(alice);
        registry.register("s1", bytes32(0));
        vm.prank(alice);
        registry.updateBatch(0, bob, "ns1", bytes32(0));
        vm.prank(bob);
        registry.update(50, bob, "entry", bytes32(0));
        assertEq(registry.getNameserverConfig(50).nameServer, "entry");
    }

    function test_EstimatedWaitTime_ReturnsZeroAtStart() public view {
        assertEq(registry.estimatedWaitTime(), 0);
    }

    function _exhaustBucket() internal {
        while (registry.estimatedWaitTime() == 0) {
            registry.register("s", bytes32(0));
        }
    }

    function test_EstimatedWaitTime_ReturnsPositiveWhenBucketEmpty() public {
        _exhaustBucket();
        assertGt(registry.estimatedWaitTime(), 0);
    }

    function test_EstimatedWaitTime_ReturnsZeroAfterRefill() public {
        _exhaustBucket();
        assertGt(registry.estimatedWaitTime(), 0);
        // Need BATCH_SIZE (256) tokens; refill rate = 2^20 / day ≈ 12.14 tokens/sec
        // 256 / 12.14 ≈ 21.09 sec → warp +22 sec
        vm.warp(block.timestamp + 22);
        assertEq(registry.estimatedWaitTime(), 0);
    }

    function test_Register_RevertsWhenBucketEmpty() public {
        _exhaustBucket();
        vm.expectRevert(abi.encodeWithSelector(MNSRegistry.RateLimit.selector, 22));
        registry.register("s", bytes32(0));
    }

    function test_Register_RefillsForRegistrationAfterElapsed() public {
        _exhaustBucket();
        // Need BATCH_SIZE (256) tokens; refill time = 256 * 86400 / 2^20 ≈ 21.09 seconds
        vm.warp(block.timestamp + 22);
        registry.register("s", bytes32(0));
    }

    function test_Register_BucketCapsAtCapacity() public {
        _exhaustBucket();
        // Warp a full day — bucket should refill to BUCKET_CAPACITY (512), no more.
        vm.warp(block.timestamp + 1 days);
        _exhaustBucket();
        vm.expectRevert(abi.encodeWithSelector(MNSRegistry.RateLimit.selector, 22));
        registry.register("s", bytes32(0));
    }

    function test_Register_RefillsAfterBurstAcrossBlocks() public {
        // Bucket starts at BUCKET_CAPACITY (512 tokens = 2 batch registrations)
        assertEq(registry.estimatedWaitTime(), 0);

        // Fully exhaust the burst
        _exhaustBucket();
        assertGt(registry.estimatedWaitTime(), 0);

        // Fast-forward ~22 seconds — enough for BATCH_SIZE (256) tokens at ~12.14/sec
        vm.warp(block.timestamp + 22);

        // Bucket refilled above the threshold — can register again
        assertEq(registry.estimatedWaitTime(), 0);
        registry.register("s", bytes32(0));
    }

    function test_ConsumeBucketToken_CarriesOverFractionalAccrual() public {
        // After the first refill window, _lastRefill should advance by less than
        // the full elapsed time so fractional token accrual carries forward.
        //
        // Without carryover (old _lastRefill = block.timestamp), warping 20s after
        // the register below would give elapsed=20 → 242 tokens → bucket=252,
        // not enough to register. With carryover, _lastRefill lags 1s behind,
        // giving elapsed=21 → 254 tokens → bucket=264 — enough to register.

        // 1. Exhaust: bucket = 0, _lastRefill = T0
        _exhaustBucket();

        // 2. Warp 22s → accrued = 22 * 1048576 / 86400 = 266 (truncated).
        //    Register → consumes 256, bucket = 10.
        //    _lastRefill += (266 * 86400) / 1048576 = 21 → _lastRefill = T0 + 21
        vm.warp(block.timestamp + 22);
        registry.register("s", bytes32(0));

        // 3. Warp 20s (total T0 + 42).
        //    With carryover:  elapsed = 42 − 21 = 21  → accrued = 254 → bucket = 264
        //    Without carryover: elapsed = 42 − 22 = 20 → accrued = 242 → bucket = 252
        vm.warp(block.timestamp + 20);

        // 4. Carryover makes the difference — registration succeeds.
        assertEq(registry.estimatedWaitTime(), 0);
        registry.register("s", bytes32(0));
    }

    function test_Entry_SetsAndUpdatesSignerHash() public {
        vm.prank(alice);
        registry.register("s1", bytes32(0));
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
        registry.register("s1", bytes32(0));
        bytes32 hashVal = keccak256("signer");
        vm.prank(alice);
        registry.update(50, alice, "custom", hashVal);
        MNSRegistry.NameserverConfig memory cfg = registry.getNameserverConfig(50);
        assertEq(cfg.nameServer, "custom");
        assertEq(cfg.signerHash, hashVal);
    }

    function test_GetNameserverConfig_SignerHashIsZeroForBatch() public {
        vm.prank(alice);
        registry.register("s1", bytes32(0));
        MNSRegistry.NameserverConfig memory cfg = registry.getNameserverConfig(0);
        assertEq(cfg.nameServer, "s1");
        assertEq(cfg.signerHash, bytes32(0));
    }

    function test_Register_RevertsWhenNameServerEmpty() public {
        vm.prank(alice);
        vm.expectRevert("empty nameserver");
        registry.register("", bytes32(0));
    }

    function test_Update_RevertsWhenNameServerEmpty() public {
        vm.prank(alice);
        registry.register("s1", bytes32(0));
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
