# Ternary Signaling — Signaling Games on {-1, 0, +1} Communication Channels

**Ternary Signaling** implements signaling game theory on a ternary communication channel. Senders with hidden quality (Low, Medium, High) choose signals {-1, 0, +1} at a cost, and receivers respond {Reject, Wait, Accept}. The crate provides Spence-style costly signaling (where higher-quality senders pay less for honest signals), cheap-talk (cost-free) baselines, and separating/pooling equilibrium analysis.

## Why It Matters

Signaling games explain how honest communication is possible when interests conflict. The ternary channel {-1, 0, +1} maps perfectly to real communication: positive endorsement (+1), neutral/no-comment (0), and negative review (-1). Costly signaling theory (Spence 1973) explains why these signals can be trusted: high-quality agents can afford to send costly positive signals that low-quality agents cannot. This is directly relevant to fleet agent reputation: when an agent endorses (+1) or rejects (-1) a proposal, the cost structure determines whether the signal is honest. The ternary neutral state (0) is uniquely important — it's the "no comment" that separates costly signaling from cheap talk.

## How It Works

### Cost Structure

Following Spence's education-as-signal model, cost is inversely related to quality:

```
Spence costs:    Low:    [0.5, 1.0, 2.0]   (positive signal is expensive)
                 Medium: [0.5, 0.5, 1.0]
                 High:   [0.5, 0.2, 0.3]   (positive signal is cheap)
```

Arrays are indexed as [negative, neutral, positive]. High-quality agents pay only 0.3 for a positive signal; low-quality agents pay 2.0. This cost differential creates a **separating equilibrium**: low-quality agents won't mimic high-quality ones because the cost exceeds the benefit.

### Cheap Talk

When costs are zero (all signals free), signals carry no information. This is the **pooling equilibrium**: all qualities send the same signal, and receivers ignore it. The cost-free baseline demonstrates why cost is necessary for honest signaling.

### Game Play

`play_round(quality, sender_strategy, receiver_strategy, costs)`:
1. Sender observes own quality, emits signal per strategy
2. Receiver observes signal (not quality), emits response per strategy
3. Payoff = benefit(response, quality) - cost(signal, quality)
4. Return `GameResult` with signal, response, cost, payoff, honesty

### Equilibrium

A **separating equilibrium** exists when each quality type sends a distinct signal. The standard Spence equilibrium:
- Low → Negative signal
- Medium → Neutral signal  
- High → Positive signal

In this equilibrium, the signal perfectly reveals quality. Verification is O(games × strategies).

## Quick Start

```rust
use ternary_signaling::{Signal, Quality, CostFunction, play_round, Response};

let costs = CostFunction::spence();

// High-quality agent sends positive signal
let result = play_round(
    Quality::High,
    |_| Signal::Positive,  // always send positive
    |s| match s { Signal::Positive => Response::Accept, _ => Response::Reject },
    &costs,
);

println!("Signal: {:?}, Cost: {:.1}, Payoff: {:.1}", result.signal, result.cost, result.payoff);
```

```bash
cargo add ternary-signaling
```

## API

| Type / Function | Description |
|---|---|
| `Signal` | `Negative(-1)`, `Neutral(0)`, `Positive(1)` |
| `Quality` | `Low`, `Medium`, `High` |
| `Response` | `Reject`, `Wait`, `Accept` |
| `CostFunction` | `spence()`, `cost_free()`, `cost(quality, signal) → f64` |
| `play_round(...)` | One signaling game iteration → `GameResult` |

## Architecture Notes

Signaling games model inter-agent communication in **SuperInstance**. The ternary channel maps to fleet messages: +1 (endorse), 0 (no comment), -1 (object). The γ + η = C conservation manifests in the cost-benefit structure: honest signals have high γ (informed decisions) and low η (deception cost), while dishonest signals have low γ and high η. See [Architecture](https://github.com/SuperInstance/SuperInstance/blob/main/ARCHITECTURE.md).

## References

- Spence, A. Michael. "Job Market Signaling," *Quarterly Journal of Economics*, 87(3), 1973 — costly signaling.
- Crawford, Vincent & Sobel, Joel. "Strategic Information Transmission," *Econometrica*, 50(6), 1982 — cheap talk.
| Gibbons, Robert. *Game Theory for Applied Economists*, Princeton UP, 1992.

## License

MIT
