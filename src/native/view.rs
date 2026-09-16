use super::*;
pub struct HistoryView {
    pub receipt: Receipt,
    pub operation: Vec<u8>,
    pub signals: Vec<Signal>,
}
impl NativeNode {
    /// Strict admission for adapters that promise every requested payment leg.
    /// Legacy native replay retains its original insufficient-leg skip semantics.
    /// Resolve an existing request first: a historical retry is not a new payment.
    pub fn validate_payments(&self, operation: &Operation) -> Result<(), Error> {
        let insufficient=match operation {
            Operation::Events(events)=>economics::Ledger::prepare(self,events)?.skipped_payments,
            Operation::Pay{from,amount,..}=>*amount==0 || self.balance(from)<*amount,
            Operation::Link{..}=>false,
        };
        if insufficient{return Err(Error::Invalid("insufficient funds for complete signed payment".into()));}
        Ok(())
    }
    pub fn history_view(&self, after: Option<u64>, limit: usize) -> Result<Vec<HistoryView>, Error> {
        self.history(after, limit)?.into_iter().map(|(receipt, bytes)| {
            let signals = match decode_operation(&bytes)? {
                Operation::Events(events) => events.into_iter().filter_map(|e|match e {
                    Event::Signal(signal)=>Some(signal), Event::Intent(_)|Event::LocalCredit{..}=>None,
                }).collect(),
                Operation::Link{..}|Operation::Pay{..} => vec![self.block(receipt.height)?
                    .ok_or_else(||Error::Corrupt("missing committed operation signal".into()))?.signal],
            };
            Ok(HistoryView{receipt,operation:bytes,signals})
        }).collect()
    }
}
