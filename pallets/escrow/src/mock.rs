use crate as pallet_escrow;
use frame_support::{
	derive_impl, parameter_types,
	traits::{ConstU128, ConstU32, ConstU64},
};
use sp_runtime::BuildStorage;

type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
	pub enum Test {
		System: frame_system,
		Balances: pallet_balances,
		EscrowPallet: pallet_escrow,
	}
);

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
	type Block = Block;
	type AccountData = pallet_balances::AccountData<u128>;
}

#[derive_impl(pallet_balances::config_preludes::TestDefaultConfig)]
impl pallet_balances::Config for Test {
	type Balance = u128;
	type ExistentialDeposit = ConstU128<1>;
	type AccountStore = System;
}

parameter_types! {
	pub const MinEscrowAmount: u128 = 10;
	pub const MaxEscrowAmount: u128 = 1_000_000;
	pub const MaxDescriptionLength: u32 = 500;
	pub const MinDeadlineBlocks: u64 = 10;
	pub const MaxDeadlineBlocks: u64 = 10_000;
	pub const MaxActiveEscrowsPerUser: u32 = 100;
}

impl pallet_escrow::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type MinEscrowAmount = MinEscrowAmount;
	type MaxEscrowAmount = MaxEscrowAmount;
	type MaxDescriptionLength = MaxDescriptionLength;
	type MinDeadlineBlocks = MinDeadlineBlocks;
	type MaxDeadlineBlocks = MaxDeadlineBlocks;
	type MaxActiveEscrowsPerUser = MaxActiveEscrowsPerUser;
	type WeightInfo = ();
}

pub const ALICE: u64 = 1;
pub const BOB: u64 = 2;
pub const CHARLIE: u64 = 3;

pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut storage = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();

	pallet_balances::GenesisConfig::<Test> {
		balances: vec![
			(ALICE, 100_000),
			(BOB, 100_000),
			(CHARLIE, 100_000),
		],
	}
	.assimilate_storage(&mut storage)
	.unwrap();

	let mut ext = sp_io::TestExternalities::new(storage);
	ext.execute_with(|| System::set_block_number(1));
	ext
}