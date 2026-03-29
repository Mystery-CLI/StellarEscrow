import { test, expect } from '@playwright/test';
import { http, HttpResponse } from 'msw';

// Mock Stellar RPC and contract responses
test.describe('Trade Workflow E2E', () => {
  test.beforeEach(async ({ page }) => {
    // Mock contract client responses
    await page.route('**/rpc/**', async route => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          results: [{xdr: 'mock_invoke_response', status: 'SUCCESS'}]
        })
      });
    });
    // Mock validate_trade_form success
    await page.route('**/contract/validate_trade_form', route => route.fulfill({
      status: 200,
      body: JSON.stringify({ok: true})
    }));
    // Mock fee bps
    await page.route('**/contract/get_fee_bps', route => route.fulfill({
      status: 200,
      body: JSON.stringify({ok: 100}) // 1%
    }));
  });

  test('User journey: Create and fund trade @smoke @critical', async ({ page }) => {
    await page.goto('/');
    
    // Create trade form
    await page.click('text=Create Trade');
    await expect(page.locator('input[placeholder*="Amount"]')).toBeVisible();
    
    await page.fill('input[placeholder*="Amount"]', '1000');
    await page.fill('[data-testid="buyer-address"]', 'GBUYER123...');
    await page.click('button:has-text("Preview")');
    
    // Preview state snapshot
    await expect(page.locator('[data-testid="preview-fee"]')).toContainText('10'); // 1%
    await expect(page).toHaveScreenshot('trade-preview.png');
    
    await page.click('button:has-text("Confirm")');
    await expect(page.locator('[data-testid="success-toast"]')).toBeVisible({timeout: 5000});
    
    // Navigate to trade detail
    await page.click('text=View Trade');
    await expect(page.locator('[data-testid="trade-status"]')).toContainText('Created');
    await expect(page).toHaveScreenshot('trade-created.png');
    
    // Fund trade
    await page.click('button:has-text("Fund Trade")');
    await page.click('button:has-text("Approve USDC")'); // Mock allowance
    await page.click('button:has-text("Confirm Fund")');
    
    await expect(page.locator('[data-testid="trade-status"]')).toContainText('Funded');
    await expect(page).toHaveScreenshot('trade-funded.png');
  });

  test('Critical path: Form validation errors', async ({ page }) => {
    await page.goto('/');
    await page.click('text=Create Trade');
    
    // Zero amount error
    await page.fill('input[placeholder*="Amount"]', '0');
    await page.click('button:has-text("Preview")');
    await expect(page.locator('[data-testid="error-invalid-amount"]')).toBeVisible();
    
    // Same buyer/seller
    await page.fill('[data-testid="buyer-address"]', 'GSELLER...');
    await expect(page.locator('[data-testid="error-unauthorized"]')).toBeVisible();
    
    await expect(page).toHaveScreenshot('form-errors.png');
  });

  test('Trade detail timeline updates', async ({ page }) => {
    await page.goto('/trades/1');
    await expect(page.locator('[data-testid="timeline-created"]')).toBeVisible();
    await expect(page).toHaveScreenshot('timeline-initial.png');
  });
});
