import { useState } from 'react';
import { Routes, Route, NavLink } from 'react-router-dom';
import { Routes, Route } from 'react-router-dom';
import { AppBar, Toolbar, Typography, Button, Box, Container } from '@mui/material';
import { NavLink } from 'react-router-dom';
import Dashboard from './pages/Dashboard';
import TradeDetail from './pages/TradeDetail';
import CreateTrade from './pages/CreateTrade';
import Help from './pages/Help';
import { ErrorBoundary } from './ErrorBoundary';

export default function App() {
  const [isMenuOpen, setIsMenuOpen] = useState(false);

  return (
    <ErrorBoundary>
      <div className="app">
        <nav className="nav">
          <span className="nav-brand">StellarEscrow</span>
          <NavLink to="/" end>Dashboard</NavLink>
          <NavLink to="/trades/new">New Trade</NavLink>
          <NavLink to="/help">Help</NavLink>
          <button 
            className="nav-mobile-toggle" 
            onClick={() => setIsMenuOpen(!isMenuOpen)}
            aria-label="Toggle navigation"
            aria-expanded={isMenuOpen}
          >
            ☰
          </button>
          <div className={`nav-links ${isMenuOpen ? 'nav-links-open' : ''}`}>
            <NavLink to="/" end onClick={() => setIsMenuOpen(false)}>Dashboard</NavLink>
            <NavLink to="/trades/new" onClick={() => setIsMenuOpen(false)}>New Trade</NavLink>
          </div>
        </nav>
        <main className="main">
      <Box sx={{ display: 'flex', flexDirection: 'column', minHeight: '100vh' }}>
        <AppBar position="static" color="primary">
          <Toolbar sx={{ gap: 2 }}>
            <Typography variant="h6" sx={{ fontWeight: 700, flexGrow: 1 }}>
              StellarEscrow
            </Typography>
            <Button color="inherit" component={NavLink} to="/" end>
              Dashboard
            </Button>
            <Button color="inherit" component={NavLink} to="/trades/new">
              New Trade
            </Button>
          </Toolbar>
        </AppBar>
        <Container component="main" sx={{ flex: 1, py: 4 }}>
          <ErrorBoundary>
            <Routes>
              <Route path="/" element={<Dashboard />} />
              <Route path="/trades/new" element={<CreateTrade />} />
              <Route path="/trades/:id" element={<TradeDetail />} />
              <Route path="/help" element={<Help />} />
            </Routes>
          </ErrorBoundary>
        </Container>
      </Box>
    </ErrorBoundary>
  );
}
