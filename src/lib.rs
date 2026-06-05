#![forbid(unsafe_code)]
//! Ternary signaling games — {-1,0,+1} as communication channel.

/// Signal type in ternary channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Signal { Negative, Neutral, Positive }

impl Signal {
    pub fn from_i8(v: i8) -> Self { match v { -1 => Signal::Negative, 0 => Signal::Neutral, _ => Signal::Positive } }
    pub fn to_i8(self) -> i8 { match self { Signal::Negative => -1, Signal::Neutral => 0, Signal::Positive => 1 } }
}

/// Quality type an agent can have.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Quality { Low, Medium, High }

/// Cost function for signals — higher quality agents pay less for honest signals.
#[derive(Debug, Clone)]
pub struct CostFunction {
    pub low_cost: [f64; 3],    // [neg, neutral, pos] costs for low-quality
    pub medium_cost: [f64; 3],
    pub high_cost: [f64; 3],
}

impl CostFunction {
    /// Spence-style: cost inversely proportional to quality.
    pub fn spence() -> Self {
        Self {
            low_cost: [0.5, 1.0, 2.0],
            medium_cost: [0.5, 0.5, 1.0],
            high_cost: [0.5, 0.2, 0.3],
        }
    }

    /// No cost — signals are cheap talk.
    pub fn cost_free() -> Self {
        Self {
            low_cost: [0.0, 0.0, 0.0],
            medium_cost: [0.0, 0.0, 0.0],
            high_cost: [0.0, 0.0, 0.0],
        }
    }

    /// Cost of sending a signal for a given quality.
    pub fn cost(&self, quality: Quality, signal: Signal) -> f64 {
        let idx = match signal { Signal::Negative => 0, Signal::Neutral => 1, Signal::Positive => 2 };
        match quality {
            Quality::Low => self.low_cost[idx],
            Quality::Medium => self.medium_cost[idx],
            Quality::High => self.high_cost[idx],
        }
    }
}

/// Response from receiver given a signal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Response { Reject, Wait, Accept }

/// A sender's strategy: maps quality to signal.
pub type SenderStrategy = fn(Quality) -> Signal;

/// A receiver's strategy: maps signal to response.
pub type ReceiverStrategy = fn(Signal) -> Response;

/// Signaling game result.
#[derive(Debug, Clone)]
pub struct GameResult {
    pub signal: Signal,
    pub response: Response,
    pub cost: f64,
    pub payoff: f64,
    pub is_honest: bool,
}

/// A round of a signaling game.
pub fn play_round(
    quality: Quality,
    sender_strategy: SenderStrategy,
    receiver_strategy: ReceiverStrategy,
    costs: &CostFunction,
    response_values: &[(Quality, Response, f64)], // (quality, response, value)
) -> GameResult {
    let signal = sender_strategy(quality);
    let response = receiver_strategy(signal);
    let cost = costs.cost(quality, signal);

    // Payoff = response value - signal cost
    let value = response_values.iter()
        .filter(|(q, r, _)| *q == quality && *r == response)
        .map(|(_, _, v)| *v)
        .next()
        .unwrap_or(0.0);

    let honest_signal = match quality {
        Quality::Low => signal == Signal::Negative,
        Quality::Medium => signal == Signal::Neutral,
        Quality::High => signal == Signal::Positive,
    };

    GameResult {
        signal,
        response,
        cost,
        payoff: value - cost,
        is_honest: honest_signal,
    }
}

/// Detect if a strategy profile is a separating equilibrium.
/// In a separating equilibrium, each quality type sends a different signal.
pub fn is_separating_equilibrium(
    strategies: &[(Quality, Signal)],
    receiver: ReceiverStrategy,
    costs: &CostFunction,
) -> bool {
    // Check: all types send different signals
    let signals: Vec<Signal> = strategies.iter().map(|(_, s)| *s).collect();
    if signals.iter().collect::<std::collections::HashSet<_>>().len() != signals.len() {
        return false;
    }

    // Check: no type wants to deviate
    for (quality, signal) in strategies {
        let current_payoff = compute_payoff(*quality, *signal, receiver, costs);
        for alt_signal in [Signal::Negative, Signal::Neutral, Signal::Positive] {
            if alt_signal == *signal { continue; }
            let alt_payoff = compute_payoff(*quality, alt_signal, receiver, costs);
            if alt_payoff > current_payoff + 1e-6 {
                return false; // Profitable deviation exists
            }
        }
    }
    true
}

/// Detect if a strategy profile is a pooling equilibrium.
/// All types send the same signal.
pub fn is_pooling_equilibrium(strategies: &[(Quality, Signal)]) -> bool {
    let first = strategies.get(0).map(|(_, s)| *s);
    first.map_or(false, |s| strategies.iter().all(|&(_, sig)| sig == s))
}

fn compute_payoff(
    quality: Quality,
    signal: Signal,
    receiver: ReceiverStrategy,
    costs: &CostFunction,
) -> f64 {
    let response = receiver(signal);
    let cost = costs.cost(quality, signal);
    let value = match (quality, response) {
        (Quality::High, Response::Accept) => 3.0,
        (Quality::High, Response::Wait) => 1.0,
        (Quality::Medium, Response::Accept) => 2.0,
        (Quality::Medium, Response::Wait) => 0.5,
        (Quality::Low, Response::Accept) => 1.0,
        (Quality::Low, Response::Wait) => 0.0,
        (_, Response::Reject) => -1.0,
    };
    value - cost
}

/// Measure signal reliability over a sequence of rounds.
pub fn signal_reliability(results: &[GameResult]) -> f64 {
    if results.is_empty() { return 0.0; }
    let honest = results.iter().filter(|r| r.is_honest).count();
    honest as f64 / results.len() as f64
}

/// Measure signal decay — reliability decreasing over time.
pub fn signal_decay(results: &[GameResult], window: usize) -> Vec<f64> {
    results.chunks(window)
        .map(|chunk| signal_reliability(chunk))
        .collect()
}

/// Costly signaling effectiveness: does cost improve honesty?
pub fn costly_signaling_score(results: &[GameResult]) -> f64 {
    if results.is_empty() { return 0.0; }
    let total_cost: f64 = results.iter().map(|r| r.cost).sum();
    let total_payoff: f64 = results.iter().map(|r| r.payoff).sum();
    let reliability = signal_reliability(results);
    // Score = reliability * (payoff / cost+1) — higher is better
    reliability * (total_payoff / (total_cost + 1.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn honest_sender(q: Quality) -> Signal {
        match q { Quality::Low => Signal::Negative, Quality::Medium => Signal::Neutral, Quality::High => Signal::Positive }
    }

    fn lying_sender(q: Quality) -> Signal { Signal::Positive } // Everyone pretends to be high

    fn random_sender(q: Quality) -> Signal {
        let _ = q; Signal::Neutral // Everyone says nothing
    }

    fn trusting_receiver(s: Signal) -> Response {
        match s { Signal::Positive => Response::Accept, Signal::Neutral => Response::Wait, Signal::Negative => Response::Reject }
    }

    fn skeptical_receiver(_: Signal) -> Response { Response::Wait }

    fn response_values() -> Vec<(Quality, Response, f64)> {
        vec![
            (Quality::High, Response::Accept, 3.0), (Quality::High, Response::Wait, 1.0), (Quality::High, Response::Reject, -1.0),
            (Quality::Medium, Response::Accept, 2.0), (Quality::Medium, Response::Wait, 0.5), (Quality::Medium, Response::Reject, -1.0),
            (Quality::Low, Response::Accept, 1.0), (Quality::Low, Response::Wait, 0.0), (Quality::Low, Response::Reject, -1.0),
        ]
    }

    #[test]
    fn test_honest_signaling() {
        let result = play_round(Quality::High, honest_sender, trusting_receiver, &CostFunction::spence(), &response_values());
        assert!(result.is_honest);
        assert_eq!(result.signal, Signal::Positive);
        assert_eq!(result.response, Response::Accept);
    }

    #[test]
    fn test_lying_signal() {
        let result = play_round(Quality::Low, lying_sender, trusting_receiver, &CostFunction::spence(), &response_values());
        assert!(!result.is_honest);
        assert_eq!(result.signal, Signal::Positive); // Low quality pretending
    }

    #[test]
    fn test_cost_reduces_lying() {
        let costs = CostFunction::spence();
        let honest = play_round(Quality::Low, honest_sender, trusting_receiver, &costs, &response_values());
        let lying = play_round(Quality::Low, lying_sender, trusting_receiver, &costs, &response_values());
        // Cost of lying should be higher than honest
        assert!(lying.cost > honest.cost, "Lying should cost more: lying={} honest={}", lying.cost, honest.cost);
    }

    #[test]
    fn test_free_signals_unreliable() {
        let results: Vec<GameResult> = [Quality::Low, Quality::Medium, Quality::High].iter()
            .map(|&q| play_round(q, lying_sender, trusting_receiver, &CostFunction::cost_free(), &response_values()))
            .collect();
        assert!(!results.iter().all(|r| r.is_honest));
    }

    #[test]
    fn test_separating_equilibrium() {
        // With steep enough costs, honest signaling can be separating
        let costs = CostFunction {
            low_cost: [0.1, 3.0, 10.0],    // Low quality pays dearly for positive
            medium_cost: [0.5, 0.5, 2.0],
            high_cost: [1.0, 0.3, 0.1],     // High quality pays little for positive
        };
        let strategies = vec![
            (Quality::Low, Signal::Negative),
            (Quality::Medium, Signal::Neutral),
            (Quality::High, Signal::Positive),
        ];
        assert!(is_separating_equilibrium(&strategies, trusting_receiver, &costs));
    }

    #[test]
    fn test_pooling_equilibrium() {
        let strategies = vec![
            (Quality::Low, Signal::Positive),
            (Quality::Medium, Signal::Positive),
            (Quality::High, Signal::Positive),
        ];
        assert!(is_pooling_equilibrium(&strategies));
    }

    #[test]
    fn test_not_separating_if_pooled() {
        let strategies = vec![
            (Quality::Low, Signal::Positive),
            (Quality::Medium, Signal::Positive),
            (Quality::High, Signal::Positive),
        ];
        assert!(!is_separating_equilibrium(&strategies, trusting_receiver, &CostFunction::spence()));
    }

    #[test]
    fn test_signal_reliability() {
        let results = vec![
            GameResult { signal: Signal::Positive, response: Response::Accept, cost: 0.3, payoff: 2.7, is_honest: true },
            GameResult { signal: Signal::Positive, response: Response::Accept, cost: 2.0, payoff: 1.0, is_honest: false },
            GameResult { signal: Signal::Neutral, response: Response::Wait, cost: 0.5, payoff: 0.0, is_honest: true },
        ];
        assert!((signal_reliability(&results) - 2.0/3.0).abs() < 0.01);
    }

    #[test]
    fn test_signal_decay() {
        let results: Vec<GameResult> = (0..20).map(|i| GameResult {
            signal: Signal::Positive, response: Response::Accept, cost: 0.3, payoff: 2.7,
            is_honest: i < 10, // First half honest, second half not
        }).collect();
        let decay = signal_decay(&results, 10);
        assert_eq!(decay.len(), 2);
        assert!(decay[0] > decay[1]);
    }

    #[test]
    fn test_costly_score() {
        let costly = GameResult { signal: Signal::Positive, response: Response::Accept, cost: 0.5, payoff: 2.5, is_honest: true };
        let cheap = GameResult { signal: Signal::Positive, response: Response::Accept, cost: 0.0, payoff: 3.0, is_honest: false };
        // Costly honest should score well
        assert!(costly_signaling_score(&[costly]) > 0.0);
    }

    #[test]
    fn test_spence_cost_structure() {
        let costs = CostFunction::spence();
        // High quality should pay less for positive signal
        assert!(costs.cost(Quality::High, Signal::Positive) < costs.cost(Quality::Low, Signal::Positive));
    }
}
