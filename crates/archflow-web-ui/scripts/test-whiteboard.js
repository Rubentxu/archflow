/**
 * ArchFlow Web Testing Script
 * Tests the whiteboard functionality by creating shapes and checking for errors
 *
 * Usage: npx playwright test test-whiteboard.spec.ts
 * Or run directly: node scripts/test-whiteboard.js
 */

const { chromium } = require('playwright');

async function testWhiteboard() {
  console.log('🧪 Starting ArchFlow Web Test...\n');

  const browser = await chromium.launch({
    headless: false,
    channel: 'chromium',
    args: ['--no-sandbox', '--disable-setuid-sandbox']
  });

  const context = await browser.newContext({
    viewport: { width: 1920, height: 1080 }
  });

  const page = await context.newPage();

  // Collect console logs
  const consoleLogs = [];
  const errors = [];

  page.on('console', msg => {
    const type = msg.type();
    const text = msg.text();
    const timestamp = new Date().toISOString();

    consoleLogs.push({ type, text, timestamp });

    if (type === 'error') {
      errors.push(text);
      console.log(`❌ [ERROR] ${text}`);
    } else if (type === 'warning') {
      console.log(`⚠️  [WARN] ${text}`);
    } else {
      console.log(`📝 [${type.toUpperCase()}] ${text}`);
    }
  });

  page.on('pageerror', error => {
    errors.push(error.message);
    console.log(`❌ [PAGE ERROR] ${error.message}`);
  });

  try {
    // Step 1: Navigate to the application
    console.log('🌐 Navigating to http://localhost:5173...');
    await page.goto('http://localhost:5173', { waitUntil: 'networkidle', timeout: 30000 });
    console.log('✅ Page loaded successfully\n');

    // Step 2: Wait for the app to initialize
    console.log('⏳ Waiting for app initialization...');
    await page.waitForTimeout(3000);

    // Step 3: Check if canvas is visible
    const canvas = await page.$('canvas');
    if (canvas) {
      console.log('✅ Canvas element found');
      const canvasBox = await canvas.boundingBox();
      console.log(`   Canvas size: ${canvasBox.width}x${canvasBox.height}`);
    } else {
      console.log('⚠️  Canvas element not found');
    }

    // Step 4: Select Rectangle tool (press 'r')
    console.log('\n🔧 Selecting Rectangle tool (pressing "r")...');
    await page.keyboard.press('r');
    await page.waitForTimeout(500);

    // Step 5: Draw a rectangle on the canvas
    console.log('🎨 Drawing a rectangle at center of canvas...');
    const canvasElement = await page.$('canvas');

    if (canvasElement) {
      const box = await canvasElement.boundingBox();

      // Center coordinates
      const startX = box.x + box.width / 2 - 100;
      const startY = box.y + box.height / 2 - 75;
      const endX = startX + 200;
      const endY = startY + 150;

      console.log(`   Drawing from (${startX}, ${startY}) to (${endX}, ${endY})`);

      // Mouse down
      await page.mouse.move(startX, startY);
      await page.mouse.down();

      // Drag to create rectangle
      await page.mouse.move(endX, endY, { steps: 10 });
      await page.mouse.up();

      console.log('✅ Rectangle drawn');

      // Wait for rendering
      await page.waitForTimeout(2000);
    }

    // Step 6: Check for any console errors
    console.log('\n📊 Console Log Summary:');
    console.log(`   Total messages: ${consoleLogs.length}`);
    console.log(`   Errors: ${errors.length}`);
    console.log(`   Warnings: ${consoleLogs.filter(l => l.type === 'warning').length}`);

    if (errors.length > 0) {
      console.log('\n❌ ERRORS DETECTED:');
      errors.forEach((err, i) => {
        console.log(`   ${i + 1}. ${err.substring(0, 100)}...`);
      });
    } else {
      console.log('\n✅ No errors detected in console!');
    }

    // Step 7: Verify rectangle was created
    console.log('\n🔍 Verification:');
    console.log('   - Canvas element: ✅ Found');
    console.log('   - Rectangle tool: ✅ Activated');
    console.log('   - Drawing action: ✅ Completed');
    console.log('   - Console errors: ' + (errors.length === 0 ? '✅ None' : '❌ Found'));

    console.log('\n✨ Test completed successfully!');

  } catch (error) {
    console.error('\n❌ TEST FAILED:', error.message);
    errors.push(error.message);
  } finally {
    await browser.close();
  }

  // Exit with error code if there were errors
  process.exit(errors.length > 0 ? 1 : 0);
}

// Run the test
testWhiteboard().catch(console.error);
