import { Routes, Route, NavLink } from 'react-router-dom';
import { AppBar, Toolbar, Typography, Button, Box, Container } from '@mui/material';
import Dashboard from './pages/Dashboard';
import TradeDetail from './pages/TradeDetail';
import CreateTrade from './pages/CreateTrade';
import Register from './pages/Register';
import Login from './pages/Login';
import UserProfile from './pages/UserProfile';
import ArbitratorRegister from './pages/ArbitratorRegister';
import { ErrorBoundary } from './ErrorBoundary';

export default function App() {
  return (
    <ErrorBoundary>
      <div className="app">
        <nav className="nav">
          <span className="nav-brand">StellarEscrow</span>
          <NavLink to="/" end>Dashboard</NavLink>
          <NavLink to="/trades/new">New Trade</NavLink>
          <NavLink to="/arbitrators/register">Become Arbitrator</NavLink>
          <NavLink to="/login">Login</NavLink>
          <NavLink to="/register">Register</NavLink>
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
            <Button color="inherit" component={NavLink} to="/arbitrators/register">
              Become Arbitrator
            <Button color="inherit" component={NavLink} to="/login">
              Login
            </Button>
            <Button color="inherit" component={NavLink} to="/register">
              Register
            </Button>
          </Toolbar>
        </AppBar>

        <Container component="main" sx={{ flex: 1, py: 4 }}>
          <ErrorBoundary>
            <Routes>
              <Route path="/" element={<Dashboard />} />
              <Route path="/trades/new" element={<CreateTrade />} />
              <Route path="/trades/:id" element={<TradeDetail />} />
              <Route path="/register" element={<Register />} />
              <Route path="/login" element={<Login />} />
              <Route path="/users/:address" element={<UserProfile />} />
              <Route path="/arbitrators/register" element={<ArbitratorRegister />} />
            </Routes>
          </ErrorBoundary>
        </Container>
      </Box>
      </main>
      </div>
    </ErrorBoundary>
  );
}
