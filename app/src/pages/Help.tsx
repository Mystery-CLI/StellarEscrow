import React, { useState } from 'react';
import { Card, Input, Button } from '@stellar-escrow/components';

interface Tutorial {
  id: number;
  title: string;
  description: string;
  time: string;
  steps: string[];
}

interface Faq {
  q: string;
  a: string;
}

interface DocArticle {
  id: string;
  title: string;
  category: string;
  content: string;
}

export default function Help() {
  const [searchQuery, setSearchQuery] = useState('');
  const [activeTab, setActiveTab] = useState<'docs' | 'faq' | 'tutorials' | 'contact'>('docs');

  const [activeTutorial, setActiveTutorial] = useState<Tutorial | null>(null);
  const [tutorialStep, setTutorialStep] = useState(0);

  const [email, setEmail] = useState('');
  const [subject, setSubject] = useState('');
  const [message, setMessage] = useState('');

  const tutorials: Tutorial[] = [
    {
      id: 1,
      title: 'How to Create a Trade',
      description: 'Learn the steps to initialize a secure escrow trade.',
      time: '3 min read',
      steps: [
        '1. Navigate to the Dashboard and click "+ New Trade".',
        '2. Enter the Buyer, Seller, and Arbitrator wallet addresses.',
        '3. Enter the total amount of USDC to be placed into Escrow.',
        '4. Review the details and submit to smart contract.',
      ],
    },
    {
      id: 2,
      title: 'Understanding the Dispute Process',
      description: 'What happens when a trade goes into dispute by one of the parties involved.',
      time: '5 min view',
      steps: [
        '1. The Buyer or Seller clicks the "Dispute" button on an active trade.',
        '2. The funds are immediately frozen on the blockchain.',
        '3. An Arbitrator reviews both parties evidence.',
        '4. The Arbitrator executes a resolution transaction, unlocking the funds.',
      ],
    },
    {
      id: 3,
      title: 'Funding an Escrow',
      description: 'A detailed guide on how to fund smart contracts using USDC.',
      time: '4 min read',
      steps: [
        '1. Open the pending trade view.',
        '2. Connect your Stellar-compatible wallet.',
        '3. Approve the USDC spending limit matching the escrow amount.',
        '4. Confirm the transaction to lock your funds inside the contract.',
      ],
    },
  ];

  const faqs: Faq[] = [
    { q: 'What is StellarEscrow?', a: 'StellarEscrow is a decentralized platform for secure peer-to-peer trades using smart contracts on the Stellar network.' },
    { q: 'How long does a trade take?', a: 'Once funded, trades settle as soon as both parties confirm. Disputes may extend the process by 48-72 hours until resolved.' },
    { q: 'What happens to my funds?', a: 'Funds are locked securely in a Soroban smart contract and can only be accessed based on the agreed trade outcomes or arbitrator decision.' },
    { q: 'Can I cancel an active trade?', a: 'Trades can be canceled and refunded if they have not been funded yet or if both parties mutually agree on the cancellation.' },
  ];

  const docs: DocArticle[] = [
    { id: 'arch', category: 'Architecture', title: 'Smart Contract Architecture', content: 'Our escrow contracts are written in Rust for the Soroban environment on the Stellar network. They utilize state channels for off-chain voting and secure multi-signature release triggers.' },
    { id: 'api-1', category: 'API', title: 'REST API Authentication', content: 'All endpoints require Bearer token authentication. Retrieve a token via stellar wallet signing procedures.' },
    { id: 'dispute-policy', category: 'Policy', title: 'Dispute Resolution Policies', content: 'When a trade is marked as disputed, an authorized arbitrator account is assigned. They gather evidence digitally via the platform and execute a final verdict.' },
    { id: 'fees', category: 'General', title: 'Platform Fee Structure', content: 'Creating a trade is free. Resolving a trade successfully generally incurs a 0.5% protocol fee, while dispute mediation costs a flat 2% fee.' },
  ];

  const filteredFaqs = faqs.filter(
    (faq) =>
      faq.q.toLowerCase().includes(searchQuery.toLowerCase()) ||
      faq.a.toLowerCase().includes(searchQuery.toLowerCase())
  );

  const filteredTutorials = tutorials.filter(
    (t) =>
      t.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
      t.description.toLowerCase().includes(searchQuery.toLowerCase())
  );

  const filteredDocs = docs.filter(
    (d) =>
      d.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
      d.content.toLowerCase().includes(searchQuery.toLowerCase()) ||
      d.category.toLowerCase().includes(searchQuery.toLowerCase())
  );

  const handleContactSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    alert('Support request submitted! We will contact you soon.');
  };

  return (
    <main className="help-center-container" aria-labelledby="help-title">
      <header className="help-header">
        <h1 id="help-title">Help Center & Documentation</h1>
        <p>Find answers, learn how to use StellarEscrow, or get in touch.</p>

        <div className="help-search" role="search">
          <Input
            label="Search help content"
            placeholder="Search documentation, FAQs, or tutorials..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            aria-label="Search documentation, FAQs, or tutorials"
          />
        </div>
      </header>

      <nav className="help-tabs" role="tablist" aria-label="Help sections">
        {(['docs', 'faq', 'tutorials', 'contact'] as const).map((tab) => (
          <button
            key={tab}
            role="tab"
            aria-selected={activeTab === tab}
            aria-controls={`panel-${tab}`}
            id={`tab-${tab}`}
            className={`help-tab ${activeTab === tab ? 'help-tab-active' : ''}`}
            onClick={() => setActiveTab(tab)}
          >
            {tab === 'docs'
              ? 'Documentation'
              : tab === 'faq'
              ? 'FAQs'
              : tab === 'tutorials'
              ? 'Interactive Tutorials'
              : 'Contact Support'}
          </button>
        ))}
      </nav>

      <section className="help-content">
        {activeTab === 'docs' && (
          <div
            id="panel-docs"
            role="tabpanel"
            aria-labelledby="tab-docs"
            className="help-section"
          >
            <h2 className="help-section-title">Official Documentation</h2>

            {filteredDocs.length === 0 && (
              <p role="status">No documentation found for "{searchQuery}".</p>
            )}

            <div
              className="docs-grid"
              role="list"
              style={{
                display: 'grid',
                gap: '1.5rem',
                gridTemplateColumns: 'repeat(auto-fill, minmax(300px, 1fr))',
              }}
            >
              {filteredDocs.map((doc) => (
                <Card key={doc.id} className="doc-card" role="listitem">
                  <span
                    style={{
                      fontSize: '0.8rem',
                      color: '#0066cc',
                      textTransform: 'uppercase',
                      fontWeight: 600,
                    }}
                  >
                    {doc.category}
                  </span>
                  <h3 style={{ margin: '0.5rem 0', fontSize: '1.2rem' }}>
                    {doc.title}
                  </h3>
                  <p style={{ color: '#555', lineHeight: 1.6 }}>
                    {doc.content}
                  </p>
                </Card>
              ))}
            </div>
          </div>
        )}

        {activeTab === 'faq' && (
          <div
            id="panel-faq"
            role="tabpanel"
            aria-labelledby="tab-faq"
            className="help-section"
          >
            <h2 className="help-section-title">Frequently Asked Questions</h2>

            {filteredFaqs.length === 0 && (
              <p role="status">No results found for "{searchQuery}".</p>
            )}

            <div className="faq-grid" role="list">
              {filteredFaqs.map((faq, i) => (
                <Card key={i} className="faq-card" role="listitem">
                  <h3>{faq.q}</h3>
                  <p>{faq.a}</p>
                </Card>
              ))}
            </div>
          </div>
        )}

        {activeTab === 'tutorials' && (
          <div
            id="panel-tutorials"
            role="tabpanel"
            aria-labelledby="tab-tutorials"
            className="help-section"
          >
            <h2 className="help-section-title">Interactive Tutorials</h2>

            {activeTutorial ? (
              <Card
                className="interactive-tutorial-viewer"
                role="region"
                aria-live="polite"
              >
                <h3>{activeTutorial.title}</h3>

                <div
                  style={{
                    marginTop: '1rem',
                    marginBottom: '2rem',
                    minHeight: '80px',
                    padding: '1.5rem',
                    background: '#f9f9f9',
                    borderRadius: '8px',
                    borderLeft: '4px solid #0066cc',
                  }}
                >
                  <p style={{ fontSize: '1.1rem', fontWeight: 500 }}>
                    {activeTutorial.steps[tutorialStep]}
                  </p>
                </div>

                <div
                  style={{ display: 'flex', justifyContent: 'space-between' }}
                >
                  <span>
                    Step {tutorialStep + 1} of {activeTutorial.steps.length}
                  </span>

                  <div style={{ display: 'flex', gap: '1rem' }}>
                    <Button
                      variant="secondary"
                      onClick={() => {
                        setActiveTutorial(null);
                        setTutorialStep(0);
                      }}
                    >
                      Close
                    </Button>

                    <Button
                      variant="primary"
                      onClick={() => {
                        if (tutorialStep < activeTutorial.steps.length - 1) {
                          setTutorialStep((s) => s + 1);
                        } else {
                          setActiveTutorial(null);
                          setTutorialStep(0);
                          alert('Tutorial completed!');
                        }
                      }}
                    >
                      {tutorialStep < activeTutorial.steps.length - 1
                        ? 'Next Step'
                        : 'Finish'}
                    </Button>
                  </div>
                </div>
              </Card>
            ) : (
              <>
                {filteredTutorials.length === 0 && (
                  <p role="status">No results found for "{searchQuery}".</p>
                )}

                <div className="tutorials-grid" role="list">
                  {filteredTutorials.map((tutorial) => (
                    <Card
                      key={tutorial.id}
                      className="tutorial-card"
                      role="listitem"
                    >
                      <h3>{tutorial.title}</h3>
                      <p>{tutorial.description}</p>

                      <div className="tutorial-meta">
                        <span>⏱ {tutorial.time}</span>
                        <Button
                          variant="secondary"
                          onClick={() => {
                            setActiveTutorial(tutorial);
                            setTutorialStep(0);
                          }}
                          aria-label={`Start tutorial ${tutorial.title}`}
                        >
                          Start Tutorial
                        </Button>
                      </div>
                    </Card>
                  ))}
                </div>
              </>
            )}
          </div>
        )}

        {activeTab === 'contact' && (
          <div
            id="panel-contact"
            role="tabpanel"
            aria-labelledby="tab-contact"
            className="help-section"
          >
            <h2 className="help-section-title">Contact Support</h2>

            <Card className="contact-card">
              <form
                onSubmit={handleContactSubmit}
                className="contact-form"
                aria-label="Contact support form"
              >
                <Input
                  label="Your Email"
                  type="email"
                  value={email}
                  onChange={(e) => setEmail(e.target.value)}
                  required
                />

                <Input
                  label="Subject"
                  value={subject}
                  onChange={(e) => setSubject(e.target.value)}
                  required
                />

                <div className="input-wrapper">
                  <label htmlFor="message" className="input-label">
                    Message
                  </label>
                  <textarea
                    id="message"
                    className="input contact-textarea"
                    rows={5}
                    value={message}
                    onChange={(e) => setMessage(e.target.value)}
                    placeholder="Describe your issue in detail..."
                    required
                  />
                </div>

                <Button type="submit" variant="primary">
                  Send Request
                </Button>
              </form>
            </Card>
          </div>
        )}
      </section>
    </main>
  );
}