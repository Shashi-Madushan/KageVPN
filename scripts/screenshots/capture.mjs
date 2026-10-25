#!/usr/bin/env node
// Regenerates the screenshots in docs/images (README) and docs-site/assets/images (docs site)
// from fake demo data.
//
//   npm run screenshots
//
// Needs a Chromium-based browser (Brave, Chrome or Edge); set BROWSER_PATH to pick one.

import { spawn } from 'node:child_process';
import { copyFileSync, existsSync, mkdirSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { chromium } from 'playwright-core';
import { demoScript } from './demo-data.mjs';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..', '..');
const out = join(root, 'docs', 'images');
const siteOut = join(root, 'docs-site', 'assets', 'images');
const PORT = 1479;
const candidates = [
	process.env.BROWSER_PATH,
	'/Applications/Google Chrome.app/Contents/MacOS/Google Chrome',
	'/Applications/Brave Browser.app/Contents/MacOS/Brave Browser',
	'/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge'
].filter(Boolean);
const executablePath = candidates.find((p) => existsSync(p));
if (!executablePath) throw new Error('No Chromium-based browser found. Set BROWSER_PATH.');
mkdirSync(out, { recursive: true });
mkdirSync(siteOut, { recursive: true });

const dev = spawn('npx', ['vite', 'dev', '--port', String(PORT), '--strictPort'], { cwd: root, stdio: 'ignore' });
const stop = () => dev.kill();
process.on('exit', stop);

async function waitForServer() {
	for (let i = 0; i < 60; i++) {
		try {
			if ((await fetch(`http://localhost:${PORT}/`)).ok) return;
		} catch {}
		await new Promise((r) => setTimeout(r, 500));
	}
	throw new Error('the dev server did not start');
}

const browser = await (async () => {
	await waitForServer();
	return chromium.launch({ executablePath, headless: true });
})();

// Wraps a page capture in a macOS window: rounded corners, traffic lights, hairline border and shadow.
async function frame(png, [w, h], theme) {
	const pad = 48;
	const page = await browser.newPage({ viewport: { width: w + pad * 2, height: h + pad * 2 }, deviceScaleFactor: 2 });
	const dark = theme === 'dark';
	await page.setContent(`<style>
		html,body{margin:0;background:transparent}
		.win{position:absolute;left:${pad}px;top:${pad}px;width:${w}px;height:${h}px;border-radius:12px;overflow:hidden;
			box-shadow:0 0 0 1px ${dark ? 'rgb(255 255 255/.16)' : 'rgb(0 0 0/.18)'},0 24px 60px rgb(0 0 0/.35),0 6px 16px rgb(0 0 0/.2)}
		img{display:block;width:100%;height:100%}
		.lights{position:absolute;left:19px;top:18px;display:flex;gap:8px}
		.lights i{width:12px;height:12px;border-radius:50%;box-shadow:inset 0 0 0 .5px rgb(0 0 0/.25)}
	</style>
	<div class="win"><img src="data:image/png;base64,${png.toString('base64')}">
	<div class="lights"><i style="background:#ff5f57"></i><i style="background:#febc2e"></i><i style="background:#28c840"></i></div></div>`);
	const out = await page.screenshot({ omitBackground: true });
	await page.close();
	return out;
}

async function shot(file, { path = '/', framed = true, connected = true, withServers = true, capOffer = false, theme = 'light', tourSeen = true, size = [1280, 800], act, selector } = {}) {
	const context = await browser.newContext({ viewport: { width: size[0], height: size[1] }, deviceScaleFactor: 2, colorScheme: theme });
	const page = await context.newPage();
	await page.addInitScript(demoScript({ connected, withServers, capOffer }));
	await page.addInitScript(([t, seen]) => {
		try {
			localStorage.setItem('kage-vpn.theme', t);
			if (seen) localStorage.setItem('kage-vpn.onboarding-seen', '1');
		} catch {}
	}, [theme, tourSeen]);
	await page.goto(`http://localhost:${PORT}${path}`);
	await page.waitForTimeout(900);
	if (act) await act(page);
	await page.waitForTimeout(500);
	const target = selector ? page.locator(selector) : page;
	let png = await target.screenshot();
	if (framed) png = await frame(png, size, theme);
	writeFileSync(join(out, file), png);
	// The docs site serves its own copy of each picture.
	copyFileSync(join(out, file), join(siteOut, file));
	console.log('wrote', file);
	await context.close();
}

const nav = (label) => async (page) => {
	await page.getByRole('button', { name: label, exact: true }).first().click();
	await page.waitForTimeout(900);
};

try {
	await shot('home.png', { framed: true, size: [1280, 740], act: nav('Home') });
	await shot('home-dark.png', { size: [1280, 860], theme: 'dark' });
	await shot('servers.png', {
		act: async (page) => {
			await nav('Servers')(page);
			await page.getByRole('button', { name: 'Test all' }).click();
			await page.waitForTimeout(900);
		}
	});
	await shot('usage.png', { size: [1280, 1180], act: nav('Data usage') });
	await shot('routing.png', { size: [1280, 900], act: nav('Routing') });
	await shot('routing-lists.png', {
		size: [1280, 1000],
		act: async (page) => {
			await nav('Routing')(page);
			await page.getByRole('tab', { name: /Ready-made lists/ }).click();
		}
	});
	await shot('apps.png', { size: [1280, 900], act: nav('App controls') });
	await shot('app-cap-editor.png', {
		size: [1280, 900],
		act: async (page) => {
			await nav('App controls')(page);
			await page.getByRole('button', { name: 'Edit the data cap for Slack' }).click();
		}
	});
	// Discord has reached its cap and offers a block, which Home shows as a card.
	await shot('home-cap-offer.png', { size: [1280, 860], capOffer: true, act: nav('Home') });
	await shot('connections-apps.png', {
		act: async (page) => {
			await nav('Connections')(page);
			await page.waitForTimeout(5000);
		}
	});
	await shot('connections.png', {
		act: async (page) => {
			await nav('Connections')(page);
			await page.getByRole('tab', { name: 'Connections' }).click();
		}
	});
	await shot('diagnostics.png', {
		size: [1280, 900],
		act: async (page) => {
			await nav('Diagnostics')(page);
			await page.getByRole('button', { name: 'Run checks' }).click();
			await page.waitForTimeout(700);
			await page.getByRole('button', { name: 'Start test' }).click();
			await page.waitForTimeout(1800);
		}
	});
	await shot('engine.png', {
		size: [1280, 900],
		act: async (page) => {
			await nav('Engine')(page);
			await page.getByRole('button', { name: 'Check for updates' }).click();
			await page.waitForTimeout(700);
		}
	});
	await shot('logs.png', { act: nav('Logs') });
	await shot('settings.png', { size: [1280, 1100], act: nav('Settings') });
	// The lower half of Settings: logs, the monthly data limit and usage privacy.
	await shot('settings-usage.png', {
		size: [1280, 1100],
		act: async (page) => {
			await nav('Settings')(page);
			await page.getByText('Keep app and website history').scrollIntoViewIfNeeded();
		}
	});
	await shot('menu-bar-popover.png', { path: '/tray', framed: false, size: [320, 550] });
	// First launch with no servers: the welcome tour opens by itself.
	await shot('welcome.png', { connected: false, withServers: false, tourSeen: false, size: [1280, 860] });
	// The setup guide's per-app step: Settings, Run setup again, then Next until it shows.
	await shot('tour-apps.png', {
		size: [1280, 860],
		act: async (page) => {
			await nav('Settings')(page);
			await page.getByRole('button', { name: 'Run setup again' }).click();
			await page.waitForTimeout(400);
			for (let i = 0; i < 12 && !(await page.getByText('Control each app').isVisible()); i++) {
				await page.getByRole('button', { name: 'Next' }).click();
				await page.waitForTimeout(250);
			}
		}
	});
} finally {
	await browser.close();
	stop();
}
