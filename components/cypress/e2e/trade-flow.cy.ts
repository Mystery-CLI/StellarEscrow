describe('Trade Flow', () => {
  beforeEach(() => {
    cy.visit('/');
  });

  it('displays the escrow dashboard', () => {
    cy.get('[data-testid="dashboard"]').should('be.visible');
  });

  it('creates a new trade', () => {
    cy.get('[data-testid="create-trade-btn"]').click();
    cy.get('[data-testid="seller-address"]').type('GBUQWP3BOUZX34ULNQG23RQ6F4BVWCIYU2IYJJMTEN4D7NOXVJPPJNBE');
    cy.get('[data-testid="buyer-address"]').type('GBBD47UZQ5DYWVV4YPVYZKRYE7JQ63ERCXZLP4GDQFVRJQG5FDORBDD');
    cy.get('[data-testid="amount"]').type('100');
    cy.get('[data-testid="submit-trade"]').click();
    cy.get('[data-testid="trade-status"]').should('contain', 'created');
  });

  it('shows trade details on card click', () => {
    cy.get('[data-testid="trade-card"]').first().click();
    cy.get('[data-testid="trade-detail"]').should('be.visible');
  });
});
