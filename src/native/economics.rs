use super::*;

pub(super) struct Ledger {
    pub changes: BTreeMap<NeuronId, u64>,
    pub supply: u64,
    pub observations: Vec<(u64, u64)>,
    pub skipped_payments: bool,
}
impl Ledger {
    pub fn prepare(node: &NativeNode, events: &[Event]) -> Result<Self, Error> {
        let mut ledger = Self {
            changes: BTreeMap::new(),
            supply: node.supply,
            observations: vec![],
            skipped_payments: false,
        };
        let from = *hemera::hash(b"zheng").as_bytes();
        let to = *hemera::hash(b"pussy").as_bytes();
        for event in events {
            let mut weight = 0u64;
            if let Event::Signal(signal) = event {
                for link in &signal.links {
                    if link.from == from && link.to == to {
                        ledger.credit(node, signal.neuron, link.amount)?;
                        ledger.supply = ledger
                            .supply
                            .checked_add(link.amount)
                            .ok_or_else(|| invalid("subsidy supply overflow"))?;
                        weight = weight
                            .checked_add(link.amount)
                            .ok_or_else(|| invalid("subsidy weight overflow"))?;
                    }
                }
                for (to, amount) in &signal.delta_pi {
                    let balance = ledger.balance(node, &signal.neuron);
                    if balance >= *amount {
                        ledger.changes.insert(signal.neuron, balance - amount);
                        ledger.credit(node, *to, *amount)?;
                    } else {
                        ledger.skipped_payments = true;
                    }
                }
            }
            ledger.observations.push((ledger.supply, weight));
        }
        Ok(ledger)
    }
    pub fn balance(&self, node: &NativeNode, neuron: &NeuronId) -> u64 {
        self.changes
            .get(neuron)
            .copied()
            .unwrap_or_else(|| node.balance(neuron))
    }
    fn credit(&mut self, node: &NativeNode, neuron: NeuronId, amount: u64) -> Result<(), Error> {
        let balance = self
            .balance(node, &neuron)
            .checked_add(amount)
            .ok_or_else(|| invalid("balance overflow"))?;
        self.changes.insert(neuron, balance);
        Ok(())
    }
}
