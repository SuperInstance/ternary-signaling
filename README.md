# ternary-signaling

**Signaling games on a ternary channel. Honesty, deception, and costly signals.**

In nature, a peacock's tail is a *costly signal* — only a healthy peacock can afford to grow one. It's honest because the cost makes faking unprofitable. In economics, education is a costly signal of ability. In ternary, a signal is one of three values {-1, 0, +1}, and the question is: when can you trust what you hear?

This crate implements signaling game theory for ternary communication. Agents have hidden qualities (Low, Medium, High) and send ternary signals. The cost of sending each signal depends on the agent's quality. Equilibrium analysis reveals when honest signaling is stable and when deception pays.

## What's Inside

- **`Signal`** — the ternary message: `Negative`, `Neutral`, `Positive`
- **`Quality`** — hidden agent quality: `Low`, `Medium`, `High`
- **`CostFunction`** — cost of each signal for each quality level. `spence()` builds the classic costly-signaling model
- **`signaling_game(costs, receiver_strategy)`** — compute Nash equilibria
- **`is_separating(strategy)`** — do different qualities send different signals? (Honest signaling)
- **`is_pooling(strategy)`** — do all qualities send the same signal? (No information transmitted)
- **`receiver_best_response(signals, qualities, strategy)`** — optimal receiver strategy given observed signals

## Quick Example

```rust
use ternary_signaling::*;

// Spence-style costly signaling: high-quality agents signal cheaply
let costs = CostFunction::spence();
// Low quality pays 2.0 for Positive signal
// High quality pays 0.5 for Positive signal

// Honest (separating) equilibrium: each quality sends a different signal
let honest = vec![
    (Quality::Low, Signal::Negative),
    (Quality::Medium, Signal::Neutral),
    (Quality::High, Signal::Positive),
];
assert!(is_separating(&honest));
```

## The Deeper Truth

**With three signals, honest communication requires three quality levels.** In binary signaling, you can only separate two groups. Ternary signaling can separate three — which maps perfectly to the Low/Medium/High quality structure. The cost differential makes honesty stable: Low-quality agents *could* send the Positive signal, but it costs them 4x what it costs High-quality agents. The expense makes deception unprofitable.

The ternary channel is also the natural model for many real-world signaling situations: traffic lights (stop/caution/go), ratings (thumbs down/neutral/thumbs up), and sentiment (negative/neutral/positive). Each has a costly-signal interpretation where the cost structure determines whether the signal is honest.

**Use cases:**
- **Economics** — model education, advertising, and warranty signaling
- **Biology** — costly signaling theory (handicap principle)
- **Reputation systems** — when are ratings honest?
- **Game theory education** — the simplest multi-signal game
- **Multi-agent communication** — design honest signaling protocols

## See Also

- **ternary-trust** — trust is what you build after signals are verified
- **ternary-auction** — auctions are signaling games with bids
- **ternary-consensus** — group signaling and information aggregation
- **ternary-bus** — the channel that carries the signals

## Install

```bash
cargo add ternary-signaling
```

## License

MIT
