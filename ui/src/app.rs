use crate::form::{Currency, TradeForm};

/// Which field is currently focused in the form
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Field {
    Seller,
    Buyer,
    Amount,
    Currency,
    Arbitrator,
}

impl Field {
    const ORDER: &'static [Field] = &[
        Field::Seller,
        Field::Buyer,
        Field::Amount,
        Field::Currency,
        Field::Arbitrator,
    ];

    pub fn next(self) -> Self {
        let pos = Self::ORDER.iter().position(|f| *f == self).unwrap_or(0);
        Self::ORDER[(pos + 1) % Self::ORDER.len()]
    }

    pub fn prev(self) -> Self {
        let pos = Self::ORDER.iter().position(|f| *f == self).unwrap_or(0);
        Self::ORDER[(pos + Self::ORDER.len() - 1) % Self::ORDER.len()]
    }
}

/// Top-level screen
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Screen {
    Form,
    Preview,
    Submitted,
}

pub struct App {
    pub form: TradeForm,
    pub focused: Field,
    pub screen: Screen,
    pub should_quit: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            form: TradeForm::default(),
            focused: Field::Seller,
            screen: Screen::Form,
            should_quit: false,
        }
    }

    /// Handle a printable character input on the current text field.
    pub fn type_char(&mut self, c: char) {
        match self.focused {
            Field::Seller => self.form.seller.push(c),
            Field::Buyer => self.form.buyer.push(c),
            Field::Amount => {
                // Only allow digits and one decimal point
                if c.is_ascii_digit() || (c == '.' && !self.form.amount.contains('.')) {
                    self.form.amount.push(c);
                }
            }
            Field::Arbitrator => self.form.arbitrator.push(c),
            Field::Currency => {} // handled by left/right
        }
    }

    pub fn backspace(&mut self) {
        match self.focused {
            Field::Seller => { self.form.seller.pop(); }
            Field::Buyer => { self.form.buyer.pop(); }
            Field::Amount => { self.form.amount.pop(); }
            Field::Arbitrator => { self.form.arbitrator.pop(); }
            Field::Currency => {}
        }
    }

    pub fn cycle_currency(&mut self, forward: bool) {
        let len = Currency::ALL.len();
        if forward {
            self.form.currency_idx = (self.form.currency_idx + 1) % len;
        } else {
            self.form.currency_idx = (self.form.currency_idx + len - 1) % len;
        }
    }

    /// Try to advance to the preview screen.
    pub fn submit_form(&mut self) {
        if self.form.validate() {
            self.screen = Screen::Preview;
        }
    }

    /// Confirm from preview — mark as submitted.
    pub fn confirm(&mut self) {
        self.screen = Screen::Submitted;
    }

    pub fn back(&mut self) {
        if self.screen == Screen::Preview {
            self.screen = Screen::Form;
        }
    }
}
