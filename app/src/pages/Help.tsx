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
        '4. Review the details and submit to smart contract.'
      ]
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
        '4. The Arbitrator executes a resolution transaction, unlocking the funds.'
      ]
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
        '4. Confirm the transaction to lock your funds inside the contract.'
      ]
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

  const filteredFaqs = faqs.filter(faq => faq.q.toLowerCase().includes(searchQuery.toLowerCase()) || faq.a.toLowerCase().includes(searchQuery.toLowerCase()));
  const filteredTutorials = tutorials.filter(t => t.title.toLowerCase().includes(searchQuery.toLowerCase()) || t.description.toLowerCase().includes(searchQuery.toLowerCase()));
  const filteredDocs = docs.filter(d => d.title.toLowerCase().includes(searchQuery.toLowerCase()) || d.content.toLowerCase().includes(searchQuery.toLowerCase()) || d.category.toLowerCase().includes(searchQuery.toLowerCase()));

  const handleContactSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    alert('Support request submitted! We will contact you soon.');
  };

  return (
    <div className="help-center-container">
      <div className="help-header">
        <h1>Help Center & Documentation</h1>
        <p>Find answers, learn how to use StellarEscrow, or get in touch.</p>
        <div className="help-search">
          <Input 
            label="" 
            placeholder="Search documentation, FAQS, or tutorials..." 
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
          />
        </div>
      </div>

      <div className="help-tabs">
        <button className={`help-tab ${activeTab === 'docs' ? 'help-tab-active' : ''}`} onClick={() => setActiveTab('docs')}>Documentation</button>
        <button className={`help-tab ${activeTab === 'faq' ? 'help-tab-active' : ''}`} onClick={() => setActiveTab('faq')}>FAQs</button>
        <button className={`help-tab ${activeTab === 'tutorials' ? 'help-tab-active' : ''}`} onClick={() => setActiveTab('tutorials')}>Interactive Tutorials</button>
        <button className={`help-tab ${activeTab === 'contact' ? 'help-tab-active' : ''}`} onClick={() => setActiveTab('contact')}>Contact Support</button>
      </div>

      <div className="help-content">
        {activeTab === 'docs' && (
          <div className="help-section">
            <h2 className="help-section-title">Official Documentation</h2>
            {filteredDocs.length === 0 ? <p className="help-empty-state">No documentation found for "{searchQuery}".</p> : null}
            <div className="docs-grid" style={{ display: 'grid', gap: '1.5rem', gridTemplateColumns: 'repeat(auto-fill, minmax(300px, 1fr))' }}>
              {filteredDocs.map((doc) => (
                <Card key={doc.id} className="doc-card">
                  <span style={{ fontSize: '0.8rem', color: '#0066cc', textTransform: 'uppercase', fontWeight: 600 }}>{doc.category}</span>
                  <h3 style={{ margin: '0.5rem 0', fontSize: '1.2rem' }}>{doc.title}</h3>
                  <p style={{ color: '#555', lineHeight: 1.6 }}>{doc.content}</p>
                </Card>
              ))}
            </div>
          </div>
        )}

        {activeTab === 'faq' && (
          <div className="help-section">
            <h2 className="help-section-title">Frequently Asked Questions</h2>
            {filteredFaqs.length === 0 ? <p className="help-empty-state">No results found for "{searchQuery}".</p> : null}
            <div className="faq-grid">
              {filteredFaqs.map((faq, i) => (
                <Card key={i} className="faq-card">
                  <h3 className="faq-q">{faq.q}</h3>
                  <p className="faq-a">{faq.a}</p>
                </Card>
              ))}
            </div>
          </div>
        )}

        {activeTab === 'tutorials' && (
          <div className="help-section">
            <h2 className="help-section-title">Interactive Tutorials</h2>
            
            {activeTutorial ? (
              <Card className="interactive-tutorial-viewer">
                <h3>{activeTutorial.title}</h3>
                <div style={{ marginTop: '1rem', marginBottom: '2rem', minHeight: '80px', padding: '1.5rem', background: '#f9f9f9', borderRadius: '8px', borderLeft: '4px solid #0066cc' }}>
                  <p style={{ fontSize: '1.1rem', fontWeight: 500 }}>{activeTutorial.steps[tutorialStep]}</p>
                </div>
                
                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                  <span style={{ color: '#666', fontSize: '0.9rem' }}>Step {tutorialStep + 1} of {activeTutorial.steps.length}</span>
                  <div style={{ display: 'flex', gap: '1rem' }}>
                    <Button variant="secondary" onClick={() => { setActiveTutorial(null); setTutorialStep(0); }}>Close</Button>
                    <Button 
                      variant="primary" 
                      onClick={() => {
                        if (tutorialStep < activeTutorial.steps.length - 1) {
                          setTutorialStep(tutorialStep + 1);
                        } else {
                          setActiveTutorial(null);
                          setTutorialStep(0);
                          alert('Tutorial completed!');
                        }
                      }}
                    >
                      {tutorialStep < activeTutorial.steps.length - 1 ? 'Next Step' : 'Finish'}
                    </Button>
                  </div>
                </div>
              </Card>
            ) : (
              <>
                {filteredTutorials.length === 0 ? <p className="help-empty-state">No results found for "{searchQuery}".</p> : null}
                <div className="tutorials-grid">
                  {filteredTutorials.map((tutorial) => (
                    <Card key={tutorial.id} className="tutorial-card">
                      <h3 className="tutorial-title">{tutorial.title}</h3>
                      <p className="tutorial-desc">{tutorial.description}</p>
                      <div className="tutorial-meta">
                        <span className="tutorial-time">⏱ {tutorial.time}</span>
                        <Button 
                          variant="secondary" 
                          className="tutorial-btn"
                          onClick={() => { setActiveTutorial(tutorial); setTutorialStep(0); }}
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
          <div className="help-section">
            <h2 className="help-section-title">Contact Support</h2>
            <Card className="contact-card">
              <form onSubmit={handleContactSubmit} className="contact-form">
                <Input label="Your Email" type="email" placeholder="email@example.com" value="" onChange={() => {}} required />
                <Input label="Subject" placeholder="How can we help?" value="" onChange={() => {}} required />
                <div className="input-wrapper">
                  <label className="input-label">Message</label>
                  <textarea className="input contact-textarea" rows={5} placeholder="Describe your issue in detail..."></textarea>
                </div>
                <Button type="submit" variant="primary">Send Request</Button>
              </form>
            </Card>
          </div>
        )}
      </div>
    </div>
  );
}
