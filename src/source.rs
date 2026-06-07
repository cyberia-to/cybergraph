// ---
// tags: cybergraph, rust
// crystal-type: source
// crystal-domain: cyber
// ---
//! `BbgSource` — the inf `RelationSource` over local bbg aggregate state.
//!
//! inf resolves every base relation through `RelationSource`, so the same
//! evaluator runs over the demo graph, a bbg snapshot, or a networked
//! cybergraph unchanged. This implementation projects the public aggregate
//! state bbg holds into the relations inf queries.
//!
//! Exposed relations (column order matches the inf demo conventions so example
//! queries run unchanged):
//!
//!   particles { cid,  energy, pi_star, weight }
//!   neurons   { id,   focus,  karma,   stake  }
//!   axons     { from, to,     weight_sum      }
//!   focus     { particle, score }              -- particle → pi_star
//!   karma     { neuron,   k     }              -- neuron → karma
//!   signals   { step, neuron, link_count, height }
//!
//! Not exposed: `cyberlinks`. Individual cyberlinks are the private layer
//! (mutator set); bbg holds only the public aggregate. A query over the bbg
//! source that references `cyberlinks` sees an empty relation — the private
//! source is a separate (future) `RelationSource`, not this one.

use bbg::BbgState;
use inf_source::{RelationSource, Schema};
use inf_value::{Tuple, Value};

/// Read-only inf relation source backed by a committed bbg snapshot.
pub struct BbgSource<'a> {
    state: &'a BbgState,
}

impl<'a> BbgSource<'a> {
    pub fn new(state: &'a BbgState) -> Self {
        Self { state }
    }
}

/// u64 → inf Int. Values in bbg are small (energy, focus, counts); the cast
/// is lossless in practice and never panics.
fn int(v: u64) -> Value {
    Value::int(v as i64)
}

impl<'a> RelationSource for BbgSource<'a> {
    fn schema(&self, rel: &str) -> Option<Schema> {
        let cols: &[&str] = match rel {
            "particles" => &["cid", "energy", "pi_star", "weight"],
            "neurons"   => &["id", "focus", "karma", "stake"],
            "axons"     => &["from", "to", "weight_sum"],
            "focus"     => &["particle", "score"],
            "karma"     => &["neuron", "k"],
            "signals"   => &["step", "neuron", "link_count", "height"],
            _ => return None,
        };
        Some(Schema::new(cols))
    }

    fn scan<'b>(&'b self, rel: &str) -> Box<dyn Iterator<Item = Tuple> + 'b> {
        match rel {
            "particles" => Box::new(self.state.particles.iter().map(|(cid, r)| {
                vec![Value::hash(*cid), int(r.energy), int(r.pi_star), int(r.weight)]
            })),
            "neurons" => Box::new(self.state.neurons.iter().map(|(id, r)| {
                vec![Value::hash(*id), int(r.focus), int(r.karma), int(r.stake)]
            })),
            "axons" => Box::new(self.state.axon_edges.iter().map(move |(axon_id, (from, to))| {
                let weight = self.state.particles.get(axon_id).map_or(0, |p| p.weight);
                vec![Value::hash(*from), Value::hash(*to), int(weight)]
            })),
            "focus" => Box::new(self.state.particles.iter().map(|(cid, r)| {
                vec![Value::hash(*cid), int(r.pi_star)]
            })),
            "karma" => Box::new(self.state.neurons.iter().map(|(id, r)| {
                vec![Value::hash(*id), int(r.karma)]
            })),
            "signals" => Box::new(self.state.signals.iter().map(|(step, r)| {
                vec![int(*step), Value::hash(r.neuron), int(r.link_count as u64), int(r.block_height)]
            })),
            _ => Box::new(std::iter::empty()),
        }
    }

    fn snapshot(&self) -> Option<u64> {
        Some(self.state.height)
    }

    /// Release 0: the query path returns plain results. Lens openings over the
    /// committed root (provable queries) arrive in a later release.
    fn provable(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bbg::Bbg;

    fn neuron(seed: u8) -> [u8; 32] { [seed; 32] }
    fn particle(seed: u8) -> [u8; 32] { [seed; 32] }

    fn seeded() -> Bbg {
        let mut bbg = Bbg::new();
        bbg.state.neurons.insert(neuron(1), bbg::types::NeuronRecord { focus: 100, karma: 50, stake: 7 });
        bbg.insert(&bbg::Signal {
            neuron: neuron(1),
            links: vec![bbg::Cyberlink {
                from: particle(2), to: particle(3), token: particle(0), amount: 4, valence: 1,
            }],
            box_moves: vec![],
            height: 0,
        }).unwrap();
        bbg
    }

    #[test]
    fn schema_known_and_unknown() {
        let bbg = Bbg::new();
        let src = BbgSource::new(&bbg.state);
        assert_eq!(src.schema("particles").unwrap().arity(), 4);
        assert_eq!(src.schema("axons").unwrap().columns, vec!["from", "to", "weight_sum"]);
        assert!(src.schema("cyberlinks").is_none(), "private layer not exposed by bbg source");
    }

    #[test]
    fn scan_particles_reflects_state() {
        let bbg = seeded();
        let src = BbgSource::new(&bbg.state);
        let rows: Vec<Tuple> = src.scan("particles").collect();
        // target particle(3) has energy 4
        let target = rows.iter().find(|r| r[0] == Value::hash(particle(3))).unwrap();
        assert_eq!(target[1], Value::int(4), "energy column reflects the staked link");
    }

    #[test]
    fn scan_neurons_and_karma() {
        let bbg = seeded();
        let src = BbgSource::new(&bbg.state);
        let neurons: Vec<Tuple> = src.scan("neurons").collect();
        assert_eq!(neurons.len(), 1);
        assert_eq!(neurons[0][2], Value::int(50), "karma column");
        let karma: Vec<Tuple> = src.scan("karma").collect();
        assert_eq!(karma[0][1], Value::int(50), "karma view");
    }

    #[test]
    fn scan_axons_carries_weight() {
        let bbg = seeded();
        let src = BbgSource::new(&bbg.state);
        let axons: Vec<Tuple> = src.scan("axons").collect();
        assert_eq!(axons.len(), 1);
        assert_eq!(axons[0][0], Value::hash(particle(2)), "axon from");
        assert_eq!(axons[0][1], Value::hash(particle(3)), "axon to");
        assert_eq!(axons[0][2], Value::int(4), "weight_sum from the staked link");
    }

    #[test]
    fn unknown_relation_scans_empty() {
        let bbg = Bbg::new();
        let src = BbgSource::new(&bbg.state);
        assert_eq!(src.scan("cyberlinks").count(), 0);
    }

    #[test]
    fn snapshot_tracks_height() {
        let mut bbg = Bbg::new();
        bbg.finalize_block();
        let src = BbgSource::new(&bbg.state);
        assert_eq!(src.snapshot(), Some(1));
    }
}
