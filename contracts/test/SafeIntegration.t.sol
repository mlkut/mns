// SPDX-License-Identifier: MIT
pragma solidity ^0.8.33;

import {Test} from "forge-std/Test.sol";
import {SafeTestTools, SafeTestLib, SafeInstance, AdvancedSafeInitParams} from "safe-tools/SafeTestTools.sol";
import {Enum} from "lib/safe-contracts/contracts/interfaces/Enum.sol";
import {MNSRegistry} from "../src/MNSRegistry.sol";

contract MNSRegistryTest is Test, SafeTestTools {
    using SafeTestLib for SafeInstance;

    MNSRegistry registry;
    SafeInstance safeInstance;
    uint256[] ownerPKs;

    function setUp() public {
        registry = new MNSRegistry();

        ownerPKs = new uint256[](3);
        ownerPKs[0] = 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80;
        ownerPKs[1] = 0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d;
        ownerPKs[2] = 0x5de4111afa1a4b94908f83103eb1f1706367c2e68ca870fc3fb9a804cdab365a;

        safeInstance = _setupSafe({
            ownerPKs: ownerPKs,
            threshold: 2,
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

    function _safeRegister(string memory ns) internal returns (uint256 batchIndex) {
        batchIndex = _currentBatchCount();
        bytes memory data = abi.encodeCall(registry.register, (bytes32(0), ns));
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
    }

    function _currentBatchCount() internal view returns (uint256) {
        return registry.nextOrdinal() / 256;
    }

    function _safeAddress() internal view returns (address) {
        return address(safeInstance.safe);
    }

    // -------------------------------------------------------------------------
    // Registration tests
    // -------------------------------------------------------------------------

    function test_SafeCanRegisterBatch() public {
        assertEq(registry.estimatedWaitTime(), 0, "should be able to register");
        _safeRegister("ns1.example.com");
        assertEq(registry.getOwner(0), _safeAddress(), "Safe should be batch owner");
        MNSRegistry.ZoneConfig memory ns = registry.getZoneConfig(0);
        assertEq(ns.ns, "ns1.example.com");
    }

    // -------------------------------------------------------------------------
    // updateBatch tests
    // -------------------------------------------------------------------------

    function test_SafeCanUpdateBatchNameServer() public {
        _safeRegister("ns1.example.com");

        bytes memory data = abi.encodeCall(registry.updateBatch, (0, _safeAddress(), bytes32(0), "ns2.updated.com"));
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

        MNSRegistry.ZoneConfig memory resolved = registry.getZoneConfig(0);
        assertEq(resolved.ns, "ns2.updated.com", "ns should be updated");
        assertEq(registry.getOwner(0), _safeAddress(), "owner should be unchanged");
    }

    function test_SafeCanTransferBatchOwnership() public {
        address newOwner = makeAddr("newOwner");
        _safeRegister("ns1.example.com");

        bytes memory data = abi.encodeCall(registry.updateBatch, (0, newOwner, bytes32(0), "ns1.example.com"));
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

        assertEq(registry.getOwner(0), newOwner, "ownership should have transferred");
    }

    function test_NonOwnerCannotUpdateBatch() public {
        _safeRegister("ns1.example.com");

        address attacker = makeAddr("attacker");
        vm.prank(attacker);
        vm.expectRevert("not owner");
        registry.updateBatch(0, attacker, bytes32(0), "evil.com");
    }

    function test_SafeCannotUpdateAfterOwnershipTransfer() public {
        address newOwner = makeAddr("newOwner");
        _safeRegister("ns1.example.com");

        bytes memory transferData = abi.encodeCall(registry.updateBatch, (0, newOwner, bytes32(0), "ns1.example.com"));
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

        bytes memory retryData =
            abi.encodeCall(registry.updateBatch, (0, _safeAddress(), bytes32(0), "ns2.reclaimed.com"));

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
        uint64 ordinal = 5;

        bytes memory data = abi.encodeCall(registry.update, (ordinal, newOwner, bytes32(0), "entry.example.com"));
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

        assertEq(registry.getOwner(ordinal), newOwner);
        MNSRegistry.ZoneConfig memory resolved = registry.getZoneConfig(ordinal);
        assertEq(resolved.ns, "entry.example.com");
    }

    // -------------------------------------------------------------------------
    // Threshold enforcement
    // -------------------------------------------------------------------------

    function test_SingleSignerCannotExecute() public {
        _safeRegister("ns1.example.com");

        bytes memory data = abi.encodeCall(registry.updateBatch, (0, _safeAddress(), bytes32(0), "ns2.updated.com"));

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
