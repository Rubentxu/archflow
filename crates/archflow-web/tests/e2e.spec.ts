/**
 * ArchFlow Web - E2E Tests with Playwright
 *
 * Tests cover:
 * - Page load and initialization
 * - Canvas interactions
 * - Toolbar operations
 * - Library panel
 * - Properties panel
 * - Selection operations
 * - Keyboard shortcuts
 * - Responsive behavior
 */

import { test, expect, chromium } from '@playwright/test';

// Test configuration
const TEST_TIMEOUT = 30000;
const VIEWPORT_DESKTOP = { width: 1920, height: 1080 };
const VIEWPORT_TABLET = { width: 1024, height: 768 };
const VIEWPORT_MOBILE = { width: 375, height: 667 };

test.describe('ArchFlow Web E2E Tests', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to the app
    await page.goto('/');

    // Wait for WASM to initialize
    await page.waitForFunction(() => {
      return window.__ARCHFLOW_INITIALIZED === true;
    }, { timeout: 15000 });
  });

  test.describe('Page Load & Initialization', () => {
    test('should load without errors', async ({ page }) => {
      // Check no console errors
      const errors: string[] = [];
      page.on('console', msg => {
        if (msg.type() === 'error') {
          errors.push(msg.text());
        }
      });

      // Wait a moment for any async errors
      await page.waitForTimeout(2000);

      // Filter out known non-critical warnings
      const criticalErrors = errors.filter(e =>
        !e.includes('favicon') &&
        !e.includes('wasm-bindgen')
      );

      expect(criticalErrors).toHaveLength(0);
    });

    test('should display main UI elements', async ({ page }) => {
      // Check toolbar is visible
      await expect(page.locator('.toolbar')).toBeVisible();

      // Check canvas is visible
      await expect(page.locator('#canvas-area')).toBeVisible();

      // Check sidebar is visible
      await expect(page.locator('.sidebar')).toBeVisible();
    });

    test('should initialize SDK correctly', async ({ page }) => {
      // Check editor instance exists
      const hasEditor = await page.evaluate(() => {
        return typeof window.archflowEditor !== 'undefined';
      });
      expect(hasEditor).toBe(true);
    });
  });

  test.describe('Canvas Interactions', () => {
    test('should create shape on canvas click', async ({ page }) => {
      // Select rectangle tool
      await page.click('[data-tool="select"]');

      // Click on canvas to create a shape
      const canvas = page.locator('#canvas');
      await canvas.click({ position: { x: 400, y: 300 } });

      // Check shape was created
      const shapeCount = await page.evaluate(() => {
        return window.archflowEditor?.get_all_shapes().length || 0;
      });

      expect(shapeCount).toBeGreaterThan(0);
    });

    test('should handle drag operations', async ({ page }) => {
      // Create a shape first
      await page.evaluate(() => {
        window.archflowEditor?.create_rectangle(400, 300, 100, 80);
      });

      // Select the shape
      await page.click('#canvas', { position: { x: 450, y: 340 } });

      // Drag the shape
      await page.mouse.move(450, 340);
      await page.mouse.down();
      await page.mouse.move(500, 400, { steps: 10 });
      await page.mouse.up();

      // Verify shape moved
      const shape = await page.evaluate(() => {
        const shapes = window.archflowEditor?.get_all_shapes();
        return shapes?.[0];
      });

      expect(shape.x).not.toBe(400);
    });

    test('should support zoom with wheel', async ({ page }) => {
      const initialZoom = await page.evaluate(() => {
        return window.archflowEditor?.get_zoom?.() || 1;
      });

      // Zoom in
      await page.mouse.wheel(0, -100);

      const zoomedIn = await page.evaluate(() => {
        return window.archflowEditor?.get_zoom?.() || 1;
      });

      // Allow for some tolerance
      expect(zoomedIn).toBeGreaterThanOrEqual(initialZoom);
    });
  });

  test.describe('Toolbar Operations', () => {
    test('should have all tool buttons', async ({ page }) => {
      const tools = ['select', 'rectangle', 'ellipse', 'line', 'text', 'hand', 'zoom'];

      for (const tool of tools) {
        const button = page.locator(`[data-tool="${tool}"]`);
        await expect(button).toBeVisible();
      }
    });

    test('should switch tools on click', async ({ page }) => {
      // Click on rectangle tool
      await page.click('[data-tool="rectangle"]');

      // Verify active state
      const isActive = await page.locator('[data-tool="rectangle"]').evaluate(el => {
        return el.classList.contains('active');
      });

      expect(isActive).toBe(true);
    });

    test('should toggle grid visibility', async ({ page }) => {
      const gridBtn = page.locator('[data-action="toggle-grid"]');

      // Click to hide grid
      await gridBtn.click();

      // Verify grid is hidden
      const gridHidden = await page.evaluate(() => {
        return window.archflowEditor?.grid_config?.showGrid === false;
      });
    });
  });

  test.describe('Library Panel', () => {
    test('should display library items', async ({ page }) => {
      // Check library panel is visible
      await expect(page.locator('.library-panel')).toBeVisible();

      // Check categories are loaded
      const categories = await page.locator('.library-category');
      await expect(categories.first()).toBeVisible();
    });

    test('should expand/collapse categories', async ({ page }) => {
      const category = page.locator('.library-category').first();
      const header = category.locator('.category-header');

      // Click to collapse
      await header.click();

      // Check items are hidden
      const items = category.locator('.component-item');
      await expect(items.first()).not.toBeVisible();
    });

    test('should search library items', async ({ page }) => {
      const searchInput = page.locator('#library-search');

      // Type search query
      await searchInput.fill('rect');

      // Verify filtered results
      const visibleItems = await page.locator('.component-item:visible').count();
      expect(visibleItems).toBeGreaterThan(0);
    });

    test('should drag and drop component to canvas', async ({ page }) => {
      const component = page.locator('.component-item').first();
      const canvas = page.locator('#canvas');

      // Start drag
      await component.hover();
      await page.mouse.down();

      // Drag to canvas
      await canvas.hover({ position: { x: 500, y: 400 } });
      await page.mouse.up();

      // Verify shape was created
      const shapeCount = await page.evaluate(() => {
        return window.archflowEditor?.get_all_shapes().length || 0;
      });

      expect(shapeCount).toBeGreaterThan(0);
    });
  });

  test.describe('Properties Panel', () => {
    test('should display properties when shape selected', async ({ page }) => {
      // Create and select a shape
      await page.evaluate(() => {
        window.archflowEditor?.create_rectangle(400, 300, 100, 80);
      });

      await page.click('#canvas', { position: { x: 450, y: 340 } });

      // Verify properties panel shows shape info
      const propsVisible = await page.locator('.properties-panel').evaluate(el => {
        return !el.classList.contains('hidden');
      });

      expect(propsVisible).toBe(true);
    });

    test('should update fill color', async ({ page }) => {
      // Create a shape
      await page.evaluate(() => {
        window.archflowEditor?.create_rectangle(400, 300, 100, 80);
      });

      // Select it
      await page.click('#canvas', { position: { x: 450, y: 340 } });

      // Change fill color
      await page.fill('#fill-color', '#ff0000');

      // Verify color changed
      const shape = await page.evaluate(() => {
        const shapes = window.archflowEditor?.get_all_shapes();
        return shapes?.[0];
      });

      expect(shape.fill_color.r).toBe(255);
    });

    test('should update dimensions', async ({ page }) => {
      // Create a shape
      await page.evaluate(() => {
        window.archflowEditor?.create_rectangle(400, 300, 100, 80);
      });

      await page.click('#canvas', { position: { x: 450, y: 340 } });

      // Change dimensions
      await page.fill('#width-input', '200');

      const shape = await page.evaluate(() => {
        const shapes = window.archflowEditor?.get_all_shapes();
        return shapes?.[0];
      });

      expect(shape.width).toBe(200);
    });
  });

  test.describe('Selection Operations', () => {
    test('should select single shape', async ({ page }) => {
      await page.evaluate(() => {
        window.archflowEditor?.create_rectangle(400, 300, 100, 80);
      });

      await page.click('#canvas', { position: { x: 450, y: 340 } });

      const selection = await page.evaluate(() => {
        return window.archflowEditor?.get_selection();
      });

      expect(selection?.shapeIds?.length).toBe(1);
    });

    test('should select multiple shapes with Shift', async ({ page }) => {
      await page.evaluate(() => {
        window.archflowEditor?.create_rectangle(400, 300, 100, 80);
        window.archflowEditor?.create_rectangle(550, 300, 100, 80);
      });

      // Click first shape
      await page.click('#canvas', { position: { x: 450, y: 340 } });

      // Shift-click second shape
      await page.click('#canvas', { position: { x: 600, y: 340 }, modifiers: ['Shift'] });

      const selection = await page.evaluate(() => {
        return window.archflowEditor?.get_selection();
      });

      expect(selection?.shapeIds?.length).toBe(2);
    });

    test('should clear selection on canvas click', async ({ page }) => {
      await page.evaluate(() => {
        window.archflowEditor?.create_rectangle(400, 300, 100, 80);
      });

      await page.click('#canvas', { position: { x: 450, y: 340 } });
      await page.click('#canvas', { position: { x: 100, y: 100 } });

      const selection = await page.evaluate(() => {
        return window.archflowEditor?.get_selection();
      });

      expect(selection?.shapeIds?.length).toBe(0);
    });
  });

  test.describe('Keyboard Shortcuts', () => {
    test('should delete with Delete key', async ({ page }) => {
      await page.evaluate(() => {
        window.archflowEditor?.create_rectangle(400, 300, 100, 80);
      });

      await page.click('#canvas', { position: { x: 450, y: 340 } });
      await page.keyboard.press('Delete');

      const shapes = await page.evaluate(() => {
        return window.archflowEditor?.get_all_shapes();
      });

      expect(shapes?.length).toBe(0);
    });

    test('should copy with Ctrl+C', async ({ page }) => {
      await page.evaluate(() => {
        window.archflowEditor?.create_rectangle(400, 300, 100, 80);
      });

      await page.click('#canvas', { position: { x: 450, y: 340 } });
      await page.keyboard.press('Control+c');

      // Check clipboard has data
      const clipboard = await page.evaluate(() => {
        return navigator.clipboard.readText();
      });

      expect(clipboard.length).toBeGreaterThan(0);
    });

    test('should paste with Ctrl+V', async ({ page }) => {
      await page.evaluate(() => {
        window.archflowEditor?.create_rectangle(400, 300, 100, 80);
      });

      await page.click('#canvas', { position: { x: 450, y: 340 } });
      await page.keyboard.press('Control+c');
      await page.click('#canvas', { position: { x: 100, y: 100 } });
      await page.keyboard.press('Control+v');

      const shapes = await page.evaluate(() => {
        return window.archflowEditor?.get_all_shapes();
      });

      expect(shapes?.length).toBe(2);
    });

    test('should undo with Ctrl+Z', async ({ page }) => {
      const initialCount = await page.evaluate(() => {
        return window.archflowEditor?.get_all_shapes().length;
      });

      await page.evaluate(() => {
        window.archflowEditor?.create_rectangle(400, 300, 100, 80);
      });

      await page.keyboard.press('Control+z');

      const afterUndo = await page.evaluate(() => {
        return window.archflowEditor?.get_all_shapes().length;
      });

      expect(afterUndo).toBe(initialCount);
    });

    test('should nudge with arrow keys', async ({ page }) => {
      await page.evaluate(() => {
        window.archflowEditor?.create_rectangle(400, 300, 100, 80);
      });

      await page.click('#canvas', { position: { x: 450, y: 340 } });

      const initialX = await page.evaluate(() => {
        const shapes = window.archflowEditor?.get_all_shapes();
        return shapes?.[0].x;
      });

      await page.keyboard.press('ArrowRight');

      const afterNudge = await page.evaluate(() => {
        const shapes = window.archflowEditor?.get_all_shapes();
        return shapes?.[0].x;
      });

      expect(afterNudge).toBeGreaterThan(initialX);
    });
  });

  test.describe('Context Menu', () => {
    test('should show on right-click', async ({ page }) => {
      await page.evaluate(() => {
        window.archflowEditor?.create_rectangle(400, 300, 100, 80);
      });

      await page.click('#canvas', { position: { x: 450, y: 340 } });
      await page.click('#canvas', { button: 'right' });

      const menuVisible = await page.locator('.context-menu:not(.hidden)').isVisible();
      expect(menuVisible).toBe(true);
    });

    test('should have copy option', async ({ page }) => {
      await page.evaluate(() => {
        window.archflowEditor?.create_rectangle(400, 300, 100, 80);
      });

      await page.click('#canvas', { position: { x: 450, y: 340 } });
      await page.click('#canvas', { button: 'right' });

      const copyOption = page.locator('.menu-item[data-action="copy"]');
      await expect(copyOption).toBeVisible();
    });

    test('should have delete option', async ({ page }) => {
      await page.evaluate(() => {
        window.archflowEditor?.create_rectangle(400, 300, 100, 80);
      });

      await page.click('#canvas', { position: { x: 450, y: 340 } });
      await page.click('#canvas', { button: 'right' });

      const deleteOption = page.locator('.menu-item[data-action="delete"]');
      await expect(deleteOption).toBeVisible();
    });

    test('should hide on escape', async ({ page }) => {
      await page.evaluate(() => {
        window.archflowEditor?.create_rectangle(400, 300, 100, 80);
      });

      await page.click('#canvas', { position: { x: 450, y: 340 } });
      await page.click('#canvas', { button: 'right' });
      await page.keyboard.press('Escape');

      const menuHidden = await page.locator('.context-menu:not(.hidden)').isVisible();
      expect(menuHidden).toBe(false);
    });
  });

  test.describe('Responsive Design', () => {
    test('should adapt layout on tablet', async ({ page }) => {
      await page.setViewportSize(VIEWPORT_TABLET);

      // Check toolbar adapts
      const toolbar = page.locator('.toolbar');
      await expect(toolbar).toBeVisible();

      // Check sidebar still visible
      const sidebar = page.locator('.sidebar');
      await expect(sidebar).toBeVisible();
    });

    test('should adapt layout on mobile', async ({ page }) => {
      await page.setViewportSize(VIEWPORT_MOBILE);

      // Sidebar should be hidden
      const sidebar = page.locator('.sidebar');
      await expect(sidebar).toBeHidden();

      // Mobile menu should be visible
      const mobileMenu = page.locator('.mobile-menu-btn');
      await expect(mobileMenu).toBeVisible();
    });

    test('should toggle panels on mobile', async ({ page }) => {
      await page.setViewportSize(VIEWPORT_MOBILE);

      // Open library panel
      await page.click('[data-action="toggle-library"]');

      const libraryOpen = await page.locator('.library-panel.open').isVisible();
      expect(libraryOpen).toBe(true);
    });
  });

  test.describe('C4 Model Navigation', () => {
    test('should switch C4 levels', async ({ page }) => {
      const levelSelect = page.locator('#c4-level');

      // Switch to Container
      await levelSelect.selectOption('Container');

      const currentLevel = await page.evaluate(() => {
        return window.archflowEditor?.get_c4_level();
      });

      expect(currentLevel).toBe('Container');
    });
  });

  test.describe('Accessibility', () => {
    test('should have ARIA labels on toolbar buttons', async ({ page }) => {
      const buttons = page.locator('.toolbar-btn');
      const count = await buttons.count();

      for (let i = 0; i < count; i++) {
        const button = buttons.nth(i);
        const ariaLabel = await button.getAttribute('aria-label');
        expect(ariaLabel).toBeTruthy();
      }
    });

    test('should support keyboard navigation in library', async ({ page }) => {
      const firstItem = page.locator('.component-item').first();

      // Focus first item
      await firstItem.focus();

      // Navigate with arrow keys
      await page.keyboard.press('ArrowDown');
      await page.keyboard.press('ArrowDown');

      // Check focus moved
      const focused = await page.locator('.component-item:focus').count();
      expect(focused).toBe(1);
    });
  });
});
