fn main() {
    println!("Hello, world!");
}

//  CurrentIndex get(fn current_index): SessionIndex;
//	"cec5070d609dd3497f72bde07fc96ba072763800a36a99fdfc7c10f6415f6ee6|Session|CurrentIndex",
//	 --------------------------------++++++++++++++++++++++++++++++++
//	  32 hex len -> 16 bytes
//
//	QueuedChanged: bool;
//	"cec5070d609dd3497f72bde07fc96ba09450bfa4b96a3fa7a3c8f40da6bf32e1|Session|QueuedChanged",
//	 --------------------------------
//
//	pub Now get(fn now) build(|_| 0.into()): T::Moment;
//	"f0c365c3cf59d671eb72da0e7a4113c49f1f0515f462cdcf84e0f1d6045dfcbb|Timestamp|Now",
//	 --------------------------------
//
//	DidUpdate: bool;
//	"f0c365c3cf59d671eb72da0e7a4113c4bbd108c4899964f707fdaffb82636065|Timestamp|DidUpdate",
//	 --------------------------------
