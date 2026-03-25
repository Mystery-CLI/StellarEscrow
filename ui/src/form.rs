/// Supported currencies for trade amount
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Currency {
    Usdc,
    Xlm,
}

impl Currency {
    pub const ALL: &'static [Currency] = &[Currency::Usdc, Currency::Xlm];

    pub fn label(&self) -> &'static str {
        match self {
            Currency::Usdc => "USDC",
            Currency::Xlm => "XLM",
        }
    }
}

/// All form fields for trade creation
#[derive(Debug, Default)]
pub struct TradeForm {
    pub seller: String,
    pub buyer: String,
    pub amount: String,
    pub currency_idx: usize,
    pub arbitrator: String,
    /// Errors keyed by field name
    pub errors: Vec<(&'static str, String)>,
}

impl TradeForm {
    /// Validate all fields; populates `errors` and returns true if valid.
    pub fn validate(&mut self) -> bool {
        self.errors.clear();

        if !is_valid_stellar_address(&self.seller) {
            self.errors.push(("seller", "Invalid Stellar address".into()));
        }
        if !is_valid_stellar_address(&self.buyer) {
            self.errors.push(("buyer", "Invalid Stellar address".into()));
        }
        if self.seller == self.buyer && !self.seller.is_empty() {
            self.errors.push(("buyer", "Buyer and seller must differ".into()));
        }
        match self.amount.trim().parse::<f64>() {
            Ok(v) if v > 0.0 => {}
            _ => self.errors.push(("amount", "Amount must be a positive number".into())),
        }
        // Arbitrator is optional; validate only if provided
        if !self.arbitrator.trim().is_empty() && !is_valid_stellar_address(&self.arbitrator) {
            self.errors.push(("arbitrator", "Invalid Stellar address".into()));
        }

        self.errors.is_empty()
    }

    pub fn currency(&self) -> Currency {
        Currency::ALL[self.currency_idx]
    }

    pub fn error_for(&self, field: &str) -> Option<&str> {
        self.errors
            .iter()
            .find(|(f, _)| *f == field)
            .map(|(_, msg)| msg.as_str())
    }
}

/// Basic Stellar address validation: starts with 'G', length 56, alphanumeric.
fn is_valid_stellar_address(addr: &str) -> bool {
    let addr = addr.trim();
    addr.len() == 56
        && addr.starts_with('G')
        && addr.chars().all(|c| c.is_ascii_alphanumeric())
}

