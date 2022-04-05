use rust_decimal::prelude::*;

use crate::numbers::{Decimalable, PreciseNumber};

#[derive(Default)]
pub struct BootstrapParams {
    pub start_offset: PreciseNumber,
    pub start_time_seconds: u64,
    pub duration_seconds: u64,
}

impl BootstrapParams {
    pub fn start(&mut self, now: u64) {
        self.start_time_seconds = now;
    }

    pub fn current_offset(&self, now_seconds: u64) -> Decimal {
        // bootstrapping hasn't started
        if now_seconds <= self.start_time_seconds {
            return self.start_offset.to_decimal();
        }

        // bootstrapping has ended
        if (now_seconds.checked_sub(self.start_time_seconds).unwrap()) >= self.duration_seconds {
            return Decimal::ZERO;
        }

        let start_offset = self.start_offset.to_decimal().to_f64().unwrap();
        let x = now_seconds - self.start_time_seconds;
        let x = x.to_f64().unwrap();

        let duration = self.duration_seconds.to_f64().unwrap();

        let a = 6.0f64;
        let k = -1.0 * a / duration;
        let decay = start_offset * (k * x).exp();

        Decimal::from_f64(decay)
            .unwrap()
            .round_dp_with_strategy(6, RoundingStrategy::ToZero)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_price_boost() {
        let bp = BootstrapParams {
            start_time_seconds: 0,
            duration_seconds: 360,
            start_offset: PreciseNumber::from_decimal(Decimal::new(9, 0)),
        };

        let o = bp.current_offset(0);
        assert_eq!(o, Decimal::new(9, 0));
        let o = bp.current_offset(360);
        assert_eq!(o, Decimal::new(0, 0));

        let o = bp.current_offset(5);
        assert_eq!(o, Decimal::new(8280399, 6));

        let o = bp.current_offset(355);
        assert_eq!(o, Decimal::new(24247, 6));
    }
}
