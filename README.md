# MNS

This is a rough draft about a system aiming to help user authored short unique random names to be used in alternative dns system, like `o4dk_sfbq.mns.alt`.

## Goals

1. Short names: 8 z-base32 character (compared to 10 phone numbers).
2. Ease of archiving and indexing without sharding and republishing load due to churn.
3. Premissionless, fast, and cheap UX of creation of new names.

Both goals are related, since a bounded set of possible names (2**40 / trillion) means that the storage load doesn't increase infinetly.

Similarly, the small set of possible name force us to work on spam prevention, to prevent squatting of significant portion of the name space.

## Proposal

1. User generates a keypair (ed25519 or secp256k1).
2. User hashes the public key with a constant salt and a `nonce` until a proof of work target is achieved.
3. User hashes the hash from the previous step with another constant salt to get a `final hash`.
5. User timestamps the final hash using OpenTimestamp, and get an incomplete timestamp file.
6. The first 5 bytes (8 zbase32 characters) of the `final hash` is the user `Name`.
7. User sends the incomplete timestamp file + the public key, nonce, and signature over the DNS Packet they want to publish to a P2P gossip network (or centralized registeries or both).
8. Network verifies the PoW, and stores the data, and occasionally checks OpenTimestamps servers mentioned in the file until it gets the proof anchored to Bitcoin.
9. Once the timestamp is anchored on Bitcoin, the user can update their packets by signing and publishing new ones using the public key generated in step #1.

## Mitigated attacks

1. Squatting too many Names is mitigated by #2 above, which makes generating 1/1000th of the trillion Names too expensive, while generating only one takes couple/few minutes.
2. Racing to timestamp the same Name with a different keypair takes too much time, that it is practically impossible to do before the timestamp is anchored on Bitcoin (in few hours).
3. If the user is paranoid, they can wait until the Name is anchored on Bitcoin to make sure no one knows their key to race them to it.
4. The p2p network doesn't need sharding or churning of small nodes like a DHT, because the cost of spamming nodes is as high as generating PoW for a valid Name, and any invalid Name results in blocking the spamming node.

### Cost of Racing to collision

An attacker noticing a newly published Name needs to find a different keypair, whose public key + a nonce, hashes to a valid proof of work, and then hashes again to the same 5 bytes Name... that roughly costs `~sqrt(2 ^ 40)~ (2^20) * (2^30)` attempts assuming that the difficulty is 30 leading zeroes. This is very unlikely to be practical in the small window between OpenTimestamps batches (few hours).

### Progressive proof of work

We can change the scheme described above, to not only require a PoW _before_ the `final hash` which truncates into the name... but also add optional PoW on top of the `final hash`... and that can keep increasing in difficulty indfeinetly, as the user does some PoW every session in a lazy way.

Using this scheme, we can help the p2p network prioritize what packets to keep in their cache, if their cache is limited, so even if someone with lots of resources created millions of Names and signed packets... over time, honest users will be the one prioritized in a scarce storage network.
