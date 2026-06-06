// SPDX-License-Identifier: MIT
pragma solidity ^0.8.33;

import "forge-std/Test.sol";
import "safe-tools/SafeTestTools.sol";
import "../src/MNSRegistry.sol";

contract MNSRegistryTest is Test, SafeTestTools {
    using SafeTestLib for SafeInstance;

    MNSRegistry registry;
    SafeInstance safeInstance;
    uint256[] ownerPKs;

    function setUp() public {
        registry = new MNSRegistry();

        // Deploy a 2-of-3 Safe using the first 3 default test signers.
        // _setupSafe() derives owners from the standard "test test test ... junk"
        // mnemonic so private keys are available for signing in tests.
        ownerPKs = new uint256[](3);
        ownerPKs[0] = 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80; // anvil signer 0
        ownerPKs[1] = 0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d; // anvil signer 1
        ownerPKs[2] = 0x5de4111afa1a4b94908f83103eb1f1706367c2e68ca870fc3fb9a804cdab365a; // anvil signer 2

        safeInstance = _setupSafe({
            ownerPKs: ownerPKs,
            threshold: 2, // 2-of-3 multisig
            initialBalance: 0,
            advancedParams: AdvancedSafeInitParams({
                includeFallbackHandler: true,
                initData: "",
                saltNonce: 0,
                setupModulesCall_to: address(0),
                setupModulesCall_data: "",
                refundAmount: 0,
                refundToken: address(0),
                refundReceiver: payable(address(0))
            })
        });
    }

    // -------------------------------------------------------------------------
    // Helpers
    // -------------------------------------------------------------------------

    /// Register a batch directly from the Safe (Safe is msg.sender).
    function _safeRegister(string memory nameServer) internal returns (uint256 batchIndex) {
        batchIndex = _currentBatchCount();
        bytes memory data = abi.encodeCall(registry.register, (nameServer, bytes32(0)));
        safeInstance.execTransaction({
            to: address(registry),
            value: 0,
            data: data,
            operation: Enum.Operation.Call,
            safeTxGas: 0,
            baseGas: 0,
            gasPrice: 0,
            gasToken: address(0),
            refundReceiver: payable(address(0)),
            signatures: "" // safe-tools signs automatically with threshold signers
        });
    }

    function _currentBatchCount() internal view returns (uint256) {
        // next_ordinal() returns 0 both when empty AND when ordinal 0 is next,
        // so we probe by trying getBatch(0) — simpler to just track via next_ordinal diff.
        // We use a try/catch to detect the empty case.
        try registry.getBatch(0) returns (MNSRegistry.Batch memory) {
            uint64 nextOrd = registry.nextOrdinal();
            return nextOrd / 256; // BATCH_SIZE = 256
        } catch {
            return 0;
        }
    }

    function _safeAddress() internal view returns (address) {
        return address(safeInstance.safe);
    }

    // -------------------------------------------------------------------------
    // Registration tests
    // -------------------------------------------------------------------------

    function test_SafeCanRegisterBatch() public {
        assertTrue(registry.canRegister(), "should be able to register");

        _safeRegister("ns1.example.com");

        MNSRegistry.Batch memory r = registry.getBatch(0);
        assertEq(r.owner, _safeAddress(), "Safe should be batch owner");
        assertEq(r.ns.nameServer, "ns1.example.com");
        assertEq(r.ordinal, 0);
    }

    // -------------------------------------------------------------------------
    // updateBatch tests — core Safe scenario
    // -------------------------------------------------------------------------

    function test_SafeCanUpdateBatchNameServer() public {
        _safeRegister("ns1.example.com");

        bytes memory data = abi.encodeCall(registry.updateBatch, (0, _safeAddress(), "ns2.updated.com", bytes32(0)));
        safeInstance.execTransaction({
            to: address(registry),
            value: 0,
            data: data,
            operation: Enum.Operation.Call,
            safeTxGas: 0,
            baseGas: 0,
            gasPrice: 0,
            gasToken: address(0),
            refundReceiver: payable(address(0)),
            signatures: ""
        });

        MNSRegistry.Batch memory r = registry.getBatch(0);
        assertEq(r.ns.nameServer, "ns2.updated.com", "nameServer should be updated");
        assertEq(r.owner, _safeAddress(), "owner should be unchanged");
    }

    function test_SafeCanTransferBatchOwnership() public {
        address newOwner = makeAddr("newOwner");
        _safeRegister("ns1.example.com");

        bytes memory data = abi.encodeCall(registry.updateBatch, (0, newOwner, "ns1.example.com", bytes32(0)));
        safeInstance.execTransaction({
            to: address(registry),
            value: 0,
            data: data,
            operation: Enum.Operation.Call,
            safeTxGas: 0,
            baseGas: 0,
            gasPrice: 0,
            gasToken: address(0),
            refundReceiver: payable(address(0)),
            signatures: ""
        });

        MNSRegistry.Batch memory r = registry.getBatch(0);
        assertEq(r.owner, newOwner, "ownership should have transferred");
    }

    function test_NonOwnerCannotUpdateBatch() public {
        _safeRegister("ns1.example.com");

        address attacker = makeAddr("attacker");
        vm.prank(attacker);
        vm.expectRevert("not owner");
        registry.updateBatch(0, attacker, "evil.com", bytes32(0));
    }

    /// Verify that after ownership transfer, the old Safe can no longer update.
    function test_SafeCannotUpdateAfterOwnershipTransfer() public {
        address newOwner = makeAddr("newOwner");
        _safeRegister("ns1.example.com");

        // Transfer ownership away from the Safe.
        bytes memory transferData = abi.encodeCall(registry.updateBatch, (0, newOwner, "ns1.example.com", bytes32(0)));
        safeInstance.execTransaction({
            to: address(registry),
            value: 0,
            data: transferData,
            operation: Enum.Operation.Call,
            safeTxGas: 0,
            baseGas: 0,
            gasPrice: 0,
            gasToken: address(0),
            refundReceiver: payable(address(0)),
            signatures: ""
        });

        // Safe tries to update again — Safe's execTransaction reverts with GS013.
        bytes memory retryData =
            abi.encodeCall(registry.updateBatch, (0, _safeAddress(), "ns2.reclaimed.com", bytes32(0)));

        // Sign with threshold signers (the Safe's current owners).
        bytes32 txHash = safeInstance.safe.getTransactionHash(
            address(registry),
            0,
            retryData,
            Enum.Operation.Call,
            0,
            0,
            0,
            address(0),
            payable(address(0)),
            safeInstance.safe.nonce()
        );
        bytes memory sigs;
        for (uint256 i = 0; i < ownerPKs.length; i++) {
            (uint8 v, bytes32 r, bytes32 s) = vm.sign(ownerPKs[i], txHash);
            sigs = bytes.concat(sigs, abi.encodePacked(r, s, v));
        }

        vm.expectRevert();
        safeInstance.safe.execTransaction(
            address(registry), 0, retryData, Enum.Operation.Call, 0, 0, 0, address(0), payable(address(0)), sigs
        );
    }

    // -------------------------------------------------------------------------
    // update (per-ordinal entry) via Safe
    // -------------------------------------------------------------------------

    function test_SafeCanUpdateEntry() public {
        _safeRegister("ns1.example.com");

        address newOwner = makeAddr("entryOwner");
        uint64 ordinal = 5; // any ordinal within the batch [0, 255]

        bytes memory data = abi.encodeCall(registry.update, (ordinal, newOwner, "entry.example.com", bytes32(0)));
        safeInstance.execTransaction({
            to: address(registry),
            value: 0,
            data: data,
            operation: Enum.Operation.Call,
            safeTxGas: 0,
            baseGas: 0,
            gasPrice: 0,
            gasToken: address(0),
            refundReceiver: payable(address(0)),
            signatures: ""
        });

        MNSRegistry.Entry memory e = registry.getEntry(ordinal);
        assertEq(e.owner, newOwner);
        assertEq(e.ns.nameServer, "entry.example.com");

        // getNameServer should now resolve to the entry's server, not the batch's.
        assertEq(registry.getNameserverConfig(ordinal).nameServer, "entry.example.com");
    }

    // -------------------------------------------------------------------------
    // Threshold enforcement
    // -------------------------------------------------------------------------

    /// Confirm that a single signer (below threshold) cannot execute.
    function test_SingleSignerCannotExecute() public {
        _safeRegister("ns1.example.com");

        // Manually build a tx signed by only 1 of 3 owners and expect failure.
        // We drop to raw execTransaction to control the signature count.
        bytes memory data = abi.encodeCall(registry.updateBatch, (0, _safeAddress(), "ns2.updated.com", bytes32(0)));

        // Sign with only signer 0 (1-of-3, below threshold of 2).
        bytes32 txHash = safeInstance.safe.getTransactionHash(
            address(registry),
            0,
            data,
            Enum.Operation.Call,
            0,
            0,
            0,
            address(0),
            payable(address(0)),
            safeInstance.safe.nonce()
        );
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(ownerPKs[0], txHash);
        bytes memory sigs = abi.encodePacked(r, s, v);

        vm.expectRevert();
        safeInstance.safe.execTransaction(
            address(registry), 0, data, Enum.Operation.Call, 0, 0, 0, address(0), payable(address(0)), sigs
        );
    }
}
