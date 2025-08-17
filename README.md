# MNS

This is a rough draft about a system aiming to help user authored short unique random names to be used in alternative dns system, like `o4dk_sfbq.mns.alt`.

## Goals

1. Short names: 8 z-base32 character (compared to 10 phone numbers).
2. Ease of archiving and indexing without sharding and republishing load due to churn.
3. Premissionless, fast, and cheap UX of creation of new names.

Both goals are related, since a bounded set of possible names (2**40 / trillion) means that the storage load doesn't increase infinetly.

Similarly, the small set of possible name force us to work on spam prevention, to prevent squatting of significant portion of the name space.

## Proposal

1. Keypair Generation: User generates a keypair (ed25519 or secp256k1) as the `genesis keypair`.
2. Name Generation   : 
  - User hashes the public key with a constant `name_generation_salt` and a `name_generation_nonce` until a `name generation proof of work` target is achieved (16 bits).
  - User hashes the Proof of Work from above one more time with the same `salt` to get the `name hash`
  - First 5 bytes (40 bits) in the `name hash` is the `Name`
3. User hashes the `name hash` with a constant `timestamping_salt` and a `timestamping_nonce` until a `timestamping PoW` target is achieved (24 bits).
4. User concatenates the the `public key` and previous `nonces`, and attempts to Timestamp them on Bitcoin, possibly using OpenTimestamps servers, or manually if they can afford it.
7. User sends the incomplete timestamp file + the public key, and nonces, as well as a signature over the DNS Packet they want to publish to a P2P gossip network (or centralized registeries or both).
8. Network verifies the PoW, and stores the data, and occasionally checks OpenTimestamps servers mentioned in the file until it gets the proof anchored to Bitcoin.
9. Once the timestamp is anchored on Bitcoin, the user can update their packets by signing and publishing new ones using the public key generated in step #1.

## Mitigated attacks

1. Squatting too many Names is mitigated by step #3 above (timestamping proof of work), which makes generating 1/1000th of the trillion Names too expensive, while generating only one takes couple/few minutes in the background, after already seeing the generated name, and possibly generate another one in less than a second if they wish so.
2. A second preimage attack, to timestamp the same Name with a different keypair requires `2^16 (name generation PoW) * 2^40 = 2^56 hashes`, that takes so long it can't be done before the User successfully timestamps/anchors their genesis on Bitcoin in couple of hours window, unless the attacker is using very expensive ASICs and giving up the opportunity cost of mining Bitcoin instead.
3. If the user is paranoid, they can wait until the Name is anchored on Bitcoin to make sure no one knows their key to race them to it.
4. The p2p network doesn't need sharding or churning of small nodes like a DHT, because the cost of spamming nodes is as high as generating PoW for a valid Name, and any invalid Name results in blocking the spamming node. Furthermore if the p2p network storage is limited, the user can publish `Retension PoW` which is similar to the `timestamping PoW` but can be generated after timsetamping... and signals commitment to this `Name` to diffrentiate the user from spammers' `Names`.
