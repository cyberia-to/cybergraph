//! Preflight of one link batch, before any BBG state changes.
use super::{ApiError, Cybergraph};
use std::collections::{BTreeMap, BTreeSet};

impl Cybergraph {
    /// BBG accepts prevalidated inputs. Preserve its ordered credit/debit and
    /// saturating debit semantics while rejecting an unrepresentable credit.
    /// The temporary map contains only balances touched by this operation.
    pub(super) fn validate_bbg_batch(&self, signal: &bbg::Signal) -> Result<(), ApiError> {
        let mut nullifiers = BTreeSet::new();
        for movement in &signal.box_moves {
            if !nullifiers.insert(movement.nullifier) {
                return Err(ApiError::BbgRejected(bbg::InsertError::DoubleSpend));
            }
        }
        let mut balances = BTreeMap::new();
        for link in &signal.links {
            let to = bbg::balance_key(&link.to, &link.token);
            let credit = balances
                .entry(to)
                .or_insert_with(|| self.bbg.state.balances.get(&to).copied().unwrap_or(0));
            *credit = credit
                .checked_add(link.amount)
                .ok_or(ApiError::AmountOverflow)?;
            let from = bbg::balance_key(&link.from, &link.token);
            let debit = balances
                .entry(from)
                .or_insert_with(|| self.bbg.state.balances.get(&from).copied().unwrap_or(0));
            *debit = debit.saturating_sub(link.amount);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn link(from: bbg::Particle, to: bbg::Particle, amount: u64) -> bbg::Cyberlink {
        bbg::Cyberlink {
            from,
            to,
            token: [0; 32],
            amount,
            valence: 1,
        }
    }
    fn signal(links: Vec<bbg::Cyberlink>, box_moves: Vec<bbg::BoxMove>) -> bbg::Signal {
        bbg::Signal {
            neuron: [1; 32],
            links,
            box_moves,
            height: 0,
        }
    }

    #[test]
    fn a_nullifier_repeated_inside_one_batch_is_refused_before_any_mutation() {
        let g = Cybergraph::new();
        let movement = || bbg::BoxMove {
            nullifier: [7; 32],
            commitment: None,
        };
        g.validate_bbg_batch(&signal(vec![], vec![movement()]))
            .unwrap();
        assert!(matches!(
            g.validate_bbg_batch(&signal(vec![], vec![movement(), movement()])),
            Err(ApiError::BbgRejected(bbg::InsertError::DoubleSpend))
        ));
    }

    #[test]
    fn a_credit_the_public_balance_cannot_hold_is_refused_and_a_debit_saturates() {
        let g = Cybergraph::new();
        g.validate_bbg_batch(&signal(vec![link([2; 32], [3; 32], u64::MAX)], vec![]))
            .unwrap();
        // the second link credits the same target: together they do not fit
        assert!(matches!(
            g.validate_bbg_batch(&signal(
                vec![link([2; 32], [3; 32], u64::MAX), link([4; 32], [3; 32], 1),],
                vec![]
            )),
            Err(ApiError::AmountOverflow)
        ));
        // an empty source keeps BBG's saturating debit: no error, no underflow
        g.validate_bbg_batch(&signal(
            vec![link([2; 32], [3; 32], 5), link([2; 32], [3; 32], 5)],
            vec![],
        ))
        .unwrap();
    }
}
