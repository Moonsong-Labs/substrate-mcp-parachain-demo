use crate as pallet_escrow;
use frame::deps::{
    frame_support::{
        derive_impl, parameter_types,
        traits::{ConstU16, ConstU32, ConstU64, ConstU128, VariantCountOf},
        PalletId,
    },
    frame_system,
    sp_core::H256,
    sp_runtime::{
        traits::{BlakeTwo256, IdentityLookup},
        BuildStorage, Percent,
    },
};

type Block = frame_system::mocking::MockBlock<Test>;

frame::deps::frame_support::construct_runtime!(
    pub enum Test
    {
        System: frame_system,
        Balances: pallet_balances,
        Timestamp: pallet_timestamp,
        Escrow: pallet_escrow,
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const SS58Prefix: u16 = 42;
}

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
    type BaseCallFilter = frame::deps::frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type Nonce = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Block = Block;
    type RuntimeEvent = RuntimeEvent;
    type BlockHashCount = ConstU64<250>;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<u128>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ConstU16<42>;
    type OnSetCode = ();
    type MaxConsumers = ConstU32<16>;
}

parameter_types! {
    pub const ExistentialDeposit: u128 = 1;
}

impl pallet_balances::Config for Test {
    type Balance = u128;
    type RuntimeEvent = RuntimeEvent;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = ();
    type MaxLocks = ConstU32<50>;
    type MaxReserves = ConstU32<50>;
    type ReserveIdentifier = [u8; 8];
    type RuntimeHoldReason = ();
    type RuntimeFreezeReason = ();
    type FreezeIdentifier = ();
    type MaxFreezes = ConstU32<0>;
    type DoneSlashHandler = ();
}

impl pallet_timestamp::Config for Test {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = ConstU64<5>;
    type WeightInfo = ();
}

parameter_types! {
    pub const EscrowPalletId: PalletId = PalletId(*b"py/escro");
    pub const MinimumEscrowAmount: u128 = 10;
    pub const MaximumEscrowAmount: u128 = 1_000_000;
    pub const MinimumDeadline: u64 = 60;
    pub const MaximumDeadline: u64 = 7_776_000;
    pub const MaxDescriptionLength: u32 = 500;
    pub const MaxDisputeReasonLength: u32 = 1000;
    pub const MaxActiveEscrowsPerUser: u32 = 100;
    pub const PlatformFeePercent: Percent = Percent::from_percent(1);
    pub const DefaultArbitratorFeePercent: Percent = Percent::from_percent(2);
}

impl pallet_escrow::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type Time = Timestamp;
    type PalletId = EscrowPalletId;
    type MinimumEscrowAmount = MinimumEscrowAmount;
    type MaximumEscrowAmount = MaximumEscrowAmount;
    type MinimumDeadline = MinimumDeadline;
    type MaximumDeadline = MaximumDeadline;
    type MaxDescriptionLength = MaxDescriptionLength;
    type MaxDisputeReasonLength = MaxDisputeReasonLength;
    type MaxActiveEscrowsPerUser = MaxActiveEscrowsPerUser;
    type PlatformFeePercent = PlatformFeePercent;
    type DefaultArbitratorFeePercent = DefaultArbitratorFeePercent;
    type WeightInfo = ();
}

pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame::deps::frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();
    
    pallet_balances::GenesisConfig::<Test> {
        balances: vec![
            (1, 100_000),
            (2, 100_000),
            (3, 100_000),
            (4, 100_000),
            (5, 100_000),
        ],
        ..Default::default()
    }
    .assimilate_storage(&mut t)
    .unwrap();
    
    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| System::set_block_number(1));
    ext
}