describe('App smoke flow', () => {
  it('loads and navigates to key routes', () => {
    cy.visit('/');
    cy.contains('StellarEscrow').should('be.visible');

    cy.contains('Login').click();
    cy.url().should('include', '/login');

    cy.contains('Register').click();
    cy.url().should('include', '/register');
  });
});
