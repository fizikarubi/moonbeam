#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::pallet;
mod base64;
mod commitment;
mod error;
mod rpc;

pub use pallet::*;

extern crate alloc;

const OFFCHAIN_EXECUTION_TIME: u64 = 35_000;

#[pallet]
pub mod pallet {
	use super::error::Error;
	use super::rpc::{build_rpc_request, RpcRequestBody, RpcResponse, SubmitParam};
	use frame_support::{
		pallet_prelude::*,
		sp_runtime::offchain::{
			http::Error::{DeadlineReached, IoError, Unknown},
			Duration,
		},
	};
	use frame_system::pallet_prelude::BlockNumberFor;
	use sp_std::vec;
	use sp_std::vec::Vec;

	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_ethereum::Config {
		/// Overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		SubmittedToCelestia { height: u64 },
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		/// TODO: Use `Local Storage` API to coordinate runs of the worker.
		/// There is no guarantee for offchain workers to run on EVERY block, there might
		/// be cases where some blocks are skipped, or for some the worker runs twice (re-orgs),
		/// so the code should be able to handle that.
		/// Docs: https://github.com/JoshOrndorff/recipes/blob/master/text/off-chain-workers/storage.md
		fn offchain_worker(_block_number: BlockNumberFor<T>) {
			log::info!("==== offchain worker ==== \n");
			if let Some(eth_block) = pallet_ethereum::CurrentBlock::<T>::get() {
				// TODO: skip if the transactions block is empty?
				let encoded_block = eth_block.encode();

				// TODO: only allow the block proposer to submit the data.
				match Self::submit_blob(&encoded_block) {
					Err(e) => log::error!("Error submitting block to celestia {:?}", e),
					Ok(response) => {
						Self::deposit_event(Event::SubmittedToCelestia {
							height: response.result,
						});
						log::info!("\n==== blob posted at height: {:?} ====\n", response.result);
					}
				}
			}
		}
	}

	impl<T: Config> Pallet<T> {
		/// Fetch current price and return the result in cents.
		fn submit_blob(blob: &[u8]) -> Result<RpcResponse, Error> {
			// Set the offchain worker execution time 35s for now.
			let deadline = sp_io::offchain::timestamp()
				.add(Duration::from_millis(super::OFFCHAIN_EXECUTION_TIME));

			let param = SubmitParam::try_from(blob)?;
			let request_body = RpcRequestBody::blob_submit(param)?.try_to_string()?;
			log::info!("\n posting to celestia with: {} \n", &request_body);

			// TODO: diable jwt token
			let request = build_rpc_request("http://localhost:26658", 
				request_body,
				"eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJBbGxvdyI6WyJwdWJsaWMiLCJyZWFkIiwid3JpdGUiLCJhZG1pbiJdfQ.3K1lFMYXKH2xsbarSSGOXL6y7wiRqCRUQAdSO3uBZZU");

			// We set the deadline for sending of the request, note that awaiting response can
			// have a separate deadline. Next we send the request, before that it's also possible
			// to alter request headers or stream body content in case of non-GET requests.
			let pending = request
				.deadline(deadline)
				.send()
				.map_err(|_| Error::Http(IoError))?;

			// The request is already being processed by the host, we are free to do anything
			// else in the worker (we can send multiple concurrent requests too).
			// At some point however we probably want to check the response though,
			// so we can block current thread and wait for it to finish.
			// Note that since the request is being driven by the host, we don't have to wait
			// for the request to have it complete, we will just not read the response.
			let response = pending
				.try_wait(deadline)
				.map_err(|_| Error::Http(DeadlineReached))??;

			if response.code != 200 {
				log::warn!("Unexpected status code: {}", response.code);
				return Err(Error::Http(Unknown));
			}

			let body: Vec<u8> = response.body().collect::<Vec<u8>>();

			let response =
				serde_json::from_slice::<RpcResponse>(&body).map_err(|e| Error::Serde(e))?;

			Ok(response)
		}
	}
}
