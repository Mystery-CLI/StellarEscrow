use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::app::{App, Field, Screen};
use crate::form::Currency;

pub fn draw(f: &mut Frame, app: &App) {
    match app.screen {
        Screen::Form => draw_form(f, app),
        Screen::Preview => draw_preview(f, app),
        Screen::Submitted => draw_submitted(f),
    }
}

fn draw_form(f: &mut Frame, app: &App) {
    let area = centered_rect(70, 80, f.size());

    let block = Block::default()
        .title(" New Trade ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    f.render_widget(Clear, area);
    f.render_widget(block, area);

    let inner = shrink(area, 2);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // seller
            Constraint::Length(3), // buyer
            Constraint::Length(3), // amount + currency
            Constraint::Length(3), // arbitrator
            Constraint::Length(1), // spacer
            Constraint::Length(1), // errors
            Constraint::Length(1), // spacer
            Constraint::Length(1), // hints
        ])
        .split(inner);

    render_input(f, chunks[0], "Seller Address", &app.form.seller, app.focused == Field::Seller, app.form.error_for("seller"));
    render_input(f, chunks[1], "Buyer Address", &app.form.buyer, app.focused == Field::Buyer, app.form.error_for("buyer"));
    render_amount_currency(f, chunks[2], app);
    render_input(f, chunks[3], "Arbitrator Address (optional)", &app.form.arbitrator, app.focused == Field::Arbitrator, app.form.error_for("arbitrator"));

    // Inline error summary
    if !app.form.errors.is_empty() {
        let msg = app.form.errors.iter().map(|(f, e)| format!("{}: {}", f, e)).collect::<Vec<_>>().join("  |  ");
        let err = Paragraph::new(msg).style(Style::default().fg(Color::Red));
        f.render_widget(err, chunks[5]);
    }

    let hints = Paragraph::new("Tab/Shift+Tab: navigate  ←/→: currency  Enter: preview  Esc: quit")
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(hints, chunks[7]);
}

fn render_input(f: &mut Frame, area: Rect, label: &str, value: &str, focused: bool, error: Option<&str>) {
    let border_color = if error.is_some() {
        Color::Red
    } else if focused {
        Color::Yellow
    } else {
        Color::Gray
    };

    let title = if let Some(e) = error {
        format!(" {} — {} ", label, e)
    } else {
        format!(" {} ", label)
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    let display = if focused {
        format!("{}█", value)
    } else {
        value.to_string()
    };

    let p = Paragraph::new(display).block(block);
    f.render_widget(p, area);
}

fn render_amount_currency(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(10), Constraint::Length(12)])
        .split(area);

    render_input(f, chunks[0], "Amount", &app.form.amount, app.focused == Field::Amount, app.form.error_for("amount"));

    // Currency selector
    let border_color = if app.focused == Field::Currency { Color::Yellow } else { Color::Gray };
    let block = Block::default()
        .title(" Currency ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    let spans: Vec<Span> = Currency::ALL.iter().enumerate().map(|(i, c)| {
        if i == app.form.currency_idx {
            Span::styled(format!("[{}]", c.label()), Style::default().fg(Color::Black).bg(Color::Yellow).add_modifier(Modifier::BOLD))
        } else {
            Span::styled(format!(" {} ", c.label()), Style::default().fg(Color::Gray))
        }
    }).collect();

    let p = Paragraph::new(Line::from(spans)).block(block);
    f.render_widget(p, chunks[1]);
}

fn draw_preview(f: &mut Frame, app: &App) {
    let area = centered_rect(60, 60, f.size());
    f.render_widget(Clear, area);

    let form = &app.form;
    let arbitrator = if form.arbitrator.trim().is_empty() {
        "None".to_string()
    } else {
        truncate(&form.arbitrator, 20)
    };

    let seller_str = truncate(&form.seller, 40);
    let buyer_str = truncate(&form.buyer, 40);
    let amount_str = format!("{} {}", form.amount.trim(), form.currency().label());

    let lines = vec![
        Line::from(vec![Span::styled("Trade Preview", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))]),
        Line::from(""),
        labeled("Seller",     &seller_str),
        labeled("Buyer",      &buyer_str),
        labeled("Amount",     &amount_str),
        labeled("Arbitrator", &arbitrator),
        Line::from(""),
        Line::from(vec![Span::styled("Enter: confirm  Esc: back", Style::default().fg(Color::DarkGray))]),
    ];

    let block = Block::default()
        .title(" Confirm Trade ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green));

    let p = Paragraph::new(lines).block(block).wrap(Wrap { trim: false });
    f.render_widget(p, area);
}

fn draw_submitted(f: &mut Frame) {
    let area = centered_rect(50, 30, f.size());
    f.render_widget(Clear, area);

    let lines = vec![
        Line::from(""),
        Line::from(vec![Span::styled("  Trade submitted successfully.", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))]),
        Line::from(""),
        Line::from(vec![Span::styled("  Press Esc to quit.", Style::default().fg(Color::DarkGray))]),
    ];

    let block = Block::default()
        .title(" Done ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green));

    let p = Paragraph::new(lines).block(block);
    f.render_widget(p, area);
}

// --- helpers ---

fn labeled<'a>(key: &'a str, val: &'a str) -> Line<'a> {
    Line::from(vec![
        Span::styled(format!("  {:<12} ", key), Style::default().fg(Color::Gray)),
        Span::styled(val.to_string(), Style::default().fg(Color::White)),
    ])
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max])
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(layout[1])[1]
}

fn shrink(r: Rect, margin: u16) -> Rect {
    Rect {
        x: r.x + margin,
        y: r.y + margin,
        width: r.width.saturating_sub(margin * 2),
        height: r.height.saturating_sub(margin * 2),
    }
}
