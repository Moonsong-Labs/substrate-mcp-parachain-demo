# Escrow Pallet

A Substrate pallet that implements a decentralized escrow system for secure peer-to-peer transactions.

## Overview

The Escrow Pallet facilitates trustless transactions between buyers and sellers by holding funds in escrow until predefined conditions are met. It ensures both parties fulfill their obligations without requiring mutual trust.

## Features

- **Create Escrow**: Buyers can lock funds with specified seller, amount, and deadline
- **Release Funds**: Buyers can release funds to seller upon satisfaction
- **Refund**: Sellers can initiate refunds if unable to fulfill orders
- **Reclaim**: Buyers can reclaim funds after deadline expiration
- **Platform Fees**: Configurable fees (0.5% - 2%) on successful transactions
- **Governance**: Fee adjustment through governance mechanisms

## User Flows

### For Buyers
1. Create an escrow by depositing funds and specifying the seller
2. Set a deadline for the transaction
3. Release funds when satisfied with goods/services
4. Reclaim funds if deadline passes without resolution

### For Sellers
1. Deliver goods/services knowing payment is secured
2. Receive payment when buyer releases funds
3. Initiate refund if unable to fulfill order

## Technical Specifications

### Configuration Parameters
- **MinEscrow**: 10 units (minimum escrow amount)
- **MaxEscrow**: 1,000,000 units (maximum escrow amount)
- **MaxActiveEscrows**: 100 per user
- **MinDeadline**: ~1 hour (600 blocks)
- **MaxDeadline**: ~90 days (1,296,000 blocks)
- **MaxDescriptionLen**: 500 characters
- **Platform Fee**: 0.5% - 2% (configurable)

### Storage
- `Escrows`: Maps escrow IDs to escrow details
- `ActiveByUser`: Tracks active escrows per user
- `PlatformFee`: Current platform fee percentage
- `NextEscrowId`: Counter for escrow IDs

### Extrinsics
- `create_escrow(seller, amount, deadline, description)`
- `release_escrow(escrow_id)`
- `refund_escrow(escrow_id)`
- `reclaim_escrow(escrow_id)`
- `set_fee(new_fee)` - Governance only

### Events
- `EscrowCreated`: New escrow created
- `EscrowReleased`: Funds released to seller
- `EscrowRefunded`: Funds returned to buyer
- `EscrowReclaimed`: Expired escrow reclaimed
- `PlatformFeeChanged`: Fee percentage updated

## Integration

Add to your runtime's `Cargo.toml`:
```toml
[dependencies]
pallet-escrow = { path = "../pallets/escrow", default-features = false }
```

Configure in your runtime:
```rust
impl pallet_escrow::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type PalletId = EscrowPalletId;
    type MinEscrow = MinEscrow;
    type MaxEscrow = MaxEscrow;
    type MaxActiveEscrows = MaxActiveEscrows;
    type MinDeadline = MinDeadline;
    type MaxDeadline = MaxDeadline;
    type MaxDescriptionLen = MaxDescriptionLen;
    type GovernanceOrigin = EnsureRoot<AccountId>;
    type WeightInfo = pallet_escrow::weights::SubstrateWeight<Runtime>;
}
```

## Testing

Run unit tests:
```bash
cargo test -p pallet-escrow
```

Run benchmarks:
```bash
cargo build --release --features runtime-benchmarks
./target/release/node benchmark pallet --pallet pallet_escrow --extrinsic '*'
```

## Security Considerations

- Funds are held in the pallet's sovereign account
- Only authorized parties can perform actions on their escrows
- No single party can unilaterally access locked funds
- Platform fees only charged on successful releases
- Automatic safeguards prevent indefinite fund locks

## License

MIT-0